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

group!({
    name: "yolol",
    options: {},
    commands: [yolol],
});

#[command]
fn yolol(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let mut flag_tokenize = false;
    let mut flag_parse = false;
    let mut flag_code = false;
    let mut flag_ast = false;

    let mut args = {
        use serenity::framework::standard::{Args, Delimiter};
        Args::new(args.message(), &[Delimiter::Single('\n'), Delimiter::Single(' ')])
    };

    loop
    {
        let current = args.current().expect("Bad args given!");
        println!("Current arg: {}", current);

        match current
        {
            "-t" => {
                flag_tokenize = true;
                args.advance();
            },
            "-p" => {
                flag_parse = true;
                args.advance();
            },
            "-c" => {
                flag_code = true;
                args.advance();
            },
            "-a" => {
                flag_ast = true;
                args.advance();
            }
            _ => break
        }
    }

    let code_arg = args.rest();
    println!("Attempt to stringify: {:?}", code_arg);

    let code = {
        use regex::Regex;
        let code_matcher = Regex::new(r"\A(?s:\n*)```(?s:[a-z]*\n)?((?s).*)\n?```\z").expect("Code matching regex failed to compile!");

        if code_matcher.is_match(code_arg)
        {
            code_matcher.captures(code_arg).expect("Problem getting capture groups from yolol code matcher!")
                .get(1).unwrap().as_str()
        }
        else
        {
            message.channel_id.say(&context.http, format!("Your supplied code isn't properly put into a code block. Be sure to surround it with triple backticks!"))?;
            return Ok(())
        }
    };

    let backtick_sanitizer = {
        use regex::Regex;
        Regex::new(r"`").expect("Backtick sanitizer regex failed to compile!")
    };

    if backtick_sanitizer.is_match(code)
    {
        message.channel_id.say(&context.http, "Your supplied code contains some backticks! No trying to break the output code blocks ;)")?;
        return Ok(())
    }

    if flag_tokenize || flag_parse || flag_code || flag_ast
    {
        let tokens = match yoloxide::tokenizer::tokenize(String::from(code))
        {
            Ok(tokens) => tokens,
            Err(error) => {
                message.channel_id.say(&context.http, format!("Tokenizer failure: ```{}```", error))?;
                return Ok(())
            }
        };

        if flag_tokenize
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

        let output = if flag_code
        {
            let mut output = String::from("");
            for line in parsed_program.0
            {
                output += &format!("{}\n", &line);
            }
            format!("Reconstructed code:\n```{}```", output)
        }
        else if flag_ast
        {
            let cylon_root = yoloxide::types::ast::cylon_ast::Root::new(parsed_program.into());
            format!("Cylon AST of program:\n```json\n{}\n```", serde_json::to_string(&cylon_root).unwrap_or("Converting to JSON failed :(".to_owned()))
        }
        else // flag_parse
        {
            format!("Parsed program:\n```{:?}```", parsed_program)
        };

        message.channel_id.say(&context.http, output)?;
        return Ok(())
    }

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
    Ok(())
}