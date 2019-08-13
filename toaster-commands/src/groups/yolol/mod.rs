use std::convert::TryInto;

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

use yoloxide::{
    environment::Environment,
    types::{
        Token,
        VecWindow,
        ast::program::Program,
    }
};

use cylon_ast::CylonRoot;

use regex::Regex;
use lazy_static::lazy_static;

mod config;
use config::{
    YololConfig,

    InputFlag,
    YololInput,

    OutputFlag
};

group!({
    name: "yolol",
    options: {},
    commands: [yolol],
});

lazy_static! {
    static ref CODE_MATCHER: Regex = Regex::new(r"\A(?s:\n*)```(?s:[a-z]*\n)?((?s).*)\n?```\z").expect("Code matching regex failed to compile!");
    static ref BACKTICK_SANITIZER: Regex = Regex::new(r"`").expect("Backtick sanitizer regex failed to compile!");
}

fn extract_input(input: &str) -> Result<&str, &str>
{
    // The regex ensures the input was formatted into a code block and has a capture group for the text of the input
    let captures = match CODE_MATCHER.captures(input)
    {
        Some(captures) => captures,
        None => {
            return Err("Your supplied code isn't properly put into a code block. Be sure to surround it with triple backticks!")
        }
    };

    // Capture 0 is the whole thing. The input capture is the only capture, meaning it's capture 1
    match captures.get(1)
    {
        Some(capture) => Ok(capture.as_str()),
        None => Err("Something is wrong with extracting input from code block! Someone might have broken the regex we use...")
    }
}

fn output_execution(input: YololInput, env: &mut Environment) -> String
{
    let code = match input
    {
        YololInput::Yolol(code) => code,
        YololInput::CylonAst(_) => {
            return "Execution from Cylon AST not yet supported! Psst, try outputting yolol then using it as input ;)".to_owned()
        }
    };

    for line in code.lines()
    {
        yoloxide::execute_line(env, String::from(line));

        if env.error.is_empty() == false
        {
            return format!("Yolol execution encountered an error: ```{}```", env.error);
        }
    }

    format!("Output environment from execution: ```{}```", env)
}

fn output_yolol(input: YololInput) -> String
{
    match parse_yolol(input)
    {
        Ok(prog) => format!("Reconstructed code: ```{}```", prog),
        Err(e) => e
    }
    
}

fn output_cylon_ast(input: YololInput) -> String
{
    let cylon_root = match input
    {
        YololInput::CylonAst(root) => root,

        yolol => match parse_yolol(yolol) {
            Ok(prog) => CylonRoot::new(prog.into()),
            Err(e) => return e
        }
    };

    match serde_json::to_string(&cylon_root)
    {
        Ok(ast) => format!("Cylon AST of program:\n```json\n{}\n```", ast),
        Err(error) => format!("Converting AST to Cylon AST failed with error: ```{}```", error)
    }
}

fn output_ast(input: YololInput) -> String
{
    match parse_yolol(input)
    {
        Ok(prog) => format!("Parsed program: ```{:?}```", prog),
        Err(e) => e
    }
}

fn output_tokens(input: YololInput) -> String
{
    match tokenize_yolol(input)
    {
        Ok(tokens) => format!("Tokenized program: ```{:?}```", tokens),
        Err(e) => e
    }
}

fn parse_yolol(input: YololInput) -> Result<Program, String>
{
    if let YololInput::CylonAst(root) = input
    {
        return Ok(root.program.try_into()?)
    }

    let tokens = tokenize_yolol(input)?;
    let mut window = VecWindow::new(&tokens, 0);

    match yoloxide::parser::parse_program(&mut window)
    {
        Ok(prog) => Ok(prog),
        Err(error) => Err(format!("Parser failure: ```{}```", error)),
    }
}

fn tokenize_yolol(input: YololInput) -> Result<Vec<Token>, String>
{
    let code = match input
    {
        YololInput::Yolol(code) => code,
        YololInput::CylonAst(_) => {
            return Err("Can't tokenize a Cylon AST!".to_owned())
        }
    };

    match yoloxide::tokenizer::tokenize(code)
    {
        Ok(tokens) => Ok(tokens),
        Err(error) => Err(format!("Tokenizer failure: ```{}```", error)),
    }
}

// // Usage: send_error!(context, message, error)
// // Error is some expression that should evaluate to a string or &str
// macro_rules! send_error {
//     ($ctx:ident, $msg:ident, $err:expr ) => {
//         $msg.channel_id.say(&$ctx.http, $err)?;
//         return Ok(())
//     };
// }

#[command]
fn yolol(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    // Wrap the provided args in a new args struct, so we can control the delimiters used
    let mut args = {
        use serenity::framework::standard::{Args, Delimiter};
        Args::new(args.message(), &[Delimiter::Single('\n'), Delimiter::Single(' ')])
    };

    // Parse arguments into a YololConfig
    let config = match YololConfig::parse_args(&mut args)
    {
        Ok(config) => config,
        Err(error) => {
            message.channel_id.say(&context.http, error)?;
            return Ok(())
        }
    };

    // Anything after the flags is expected to be the input
    let input = match extract_input(args.rest())
    {
        Ok(input) => input,
        Err(error) => {
            message.channel_id.say(&context.http, error)?;
            return Ok(())
        }
    };

    // Quickly checks to make sure there's no backticks in the code, since they can break output formatting
    if BACKTICK_SANITIZER.is_match(input)
    {
        message.channel_id.say(&context.http, "Your supplied code contains some backticks! No trying to break the output code blocks ;)")?;
        return Ok(())
    }

    let input = match config.input
    {
        InputFlag::Yolol => YololInput::Yolol(input.to_owned()),

        InputFlag::CylonAst => match serde_json::from_str(input) {
            Ok(root) => YololInput::CylonAst(root),
            Err(error) => {
                message.channel_id.say(&context.http, format!("Converting Cylon AST json to internal representation failed with error: ```{}```", error))?;
                return Ok(())
            }
        },
    };

    println!("Output: {:?}, input: {:?}", config.input, config.output);

    match config.output
    {
        OutputFlag::Execution => {
            let mut env = Environment::new("Bot");
            message.channel_id.say(&context.http, output_execution(input, &mut env))?;
        },
        OutputFlag::Yolol => {
            message.channel_id.say(&context.http, output_yolol(input))?;
        },
        OutputFlag::CylonAst => {
            let mut output = output_cylon_ast(input);

            if output.len() > 2000
            {
                let other_half = output.split_off(1997);
                message.channel_id.say(&context.http, output + "```")?;
                message.channel_id.say(&context.http, "```json\n".to_owned() + other_half.as_str())?;
            }
            else
            {
                message.channel_id.say(&context.http, output)?;
            }

        },
        OutputFlag::Ast => {
            message.channel_id.say(&context.http, output_ast(input))?;
        },
        OutputFlag::Tokens => {
            message.channel_id.say(&context.http, output_tokens(input))?;
        }
    }

    Ok(())
}