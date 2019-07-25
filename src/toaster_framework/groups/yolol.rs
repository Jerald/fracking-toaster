use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    CommandResult,
    Args,
    macros::{
        command,
        group
    }
};

use regex::Regex;
use lazy_static::lazy_static;

group!({
    name: "yolol",
    options: {},
    commands: [yolol],
});

lazy_static! {
    static ref CODE_MATCHER: Regex = Regex::new(r"\A(?s:\n*)```(?s:[a-z]*\n)?((?s).*)\n?```\z").expect("Code matching regex failed to compile!");
    static ref BACKTICK_SANITIZER: Regex = Regex::new(r"`").expect("Backtick sanitizer regex failed to compile!");
}

struct YololConfig
{
    pub input_yolol: bool,
    pub input_cylon_ast: bool,

    pub output_execution: bool,
    pub output_yolol: bool,
    pub output_cylon_ast: bool,
    pub output_parsed: bool,
    pub output_tokenized: bool,
}

impl YololConfig
{
    fn new() -> Self
    {
        Default::default()
    }

    fn parse_args(args: &mut Args) -> Self
    {
        let mut config = YololConfig::new();

        loop
        {
            let current = args.current().expect("Bad args given!");
            println!("Current arg: {}", current);

            match current
            {
                "-c" => config.output_yolol = true,
                "-a" => config.output_cylon_ast = true,
                "-p" => config.output_parsed = true,
                "-t" => config.output_tokenized = true,

                _ => break
            }

            args.advance();
        }

        config
    }
}

impl Default for YololConfig
{
    fn default() -> Self
    {
        YololConfig {
            input_yolol: true,
            input_cylon_ast: false,

            output_execution: true,
            output_yolol: false,
            output_cylon_ast: false,
            output_parsed: false,
            output_tokenized: false,
        }
    }
}

#[command]
fn yolol(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let mut args = {
        use serenity::framework::standard::{Args, Delimiter};
        Args::new(args.message(), &[Delimiter::Single('\n'), Delimiter::Single(' ')])
    };

    let config = YololConfig::parse_args(&mut args);

    let input_arg = args.rest();
    println!("Input argument:\n{}", input_arg);

    let code = match CODE_MATCHER.captures(input_arg)
    {
        Some(captures) => captures.get(1).expect("Unable to get capture from code matcher!").as_str(),
        None => {
            message.channel_id.say(&context.http, "Your supplied code isn't properly put into a code block. Be sure to surround it with triple backticks!")?;
            return Ok(())
        }
    };

    if BACKTICK_SANITIZER.is_match(code)
    {
        message.channel_id.say(&context.http, "Your supplied code contains some backticks! No trying to break the output code blocks ;)")?;
        return Ok(())
    }

    if config.output_execution
    {
        let output_env = {
            use yoloxide::environment::Environment;
            let mut env = Environment::new("Bot execution");

            for line in code.lines()
            {
                yoloxide::execute_line(&mut env, String::from(line));

                if env.error.is_empty() == false
                {
                    message.channel_id.say(&context.http, format!("Yolol execution encountered an error: ```{}```", env.error))?;
                    return Ok(());
                }
            }

            env
        };

        message.channel_id.say(&context.http, format!("Output environment from execution: ```{}```", output_env))?;
        return Ok(());
    }

    let tokens = match yoloxide::tokenizer::tokenize(String::from(code))
    {
        Ok(tokens) => tokens,
        Err(error) => {
            message.channel_id.say(&context.http, format!("Tokenizer failure: ```{}```", error))?;
            return Ok(())
        }
    };

    if config.output_tokenized
    {
        message.channel_id.say(&context.http, format!("Tokenized program:\n```{:?}```", tokens))?;
        return Ok(())
    }

    let mut window = yoloxide::types::VecWindow::new(&tokens, 0);
    let parsed_program = match yoloxide::parser::parse_program(&mut window)
    {
        Ok(program) => program,
        Err(error) => {
            message.channel_id.say(&context.http, format!("Parser failure: ```{}```", error))?;
            return Ok(())
        }
    };

    if config.output_parsed
    {
        message.channel_id.say(&context.http, format!("Parsed program:\n```{:?}```", parsed_program))?;
    }
    else if config.output_cylon_ast
    {
        let cylon_root = yoloxide::types::ast::cylon_ast::Root::new(parsed_program.into());
        let ast_str = serde_json::to_string(&cylon_root).unwrap_or("Converting to JSON failed :(".to_owned());

        message.channel_id.say(&context.http, format!("Cylon AST of program:\n```json\n{}\n```", ast_str))?;
    }
    else if config.output_yolol
    {
        message.channel_id.say(&context.http, format!("Reconstructed code:\n```{}```", parsed_program.to_string()))?;
    }

    Ok(())
}