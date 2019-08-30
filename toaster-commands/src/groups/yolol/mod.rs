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

fn output_execution(input: YololInput, env: &mut Environment) -> Result<(), String>
{
    let tick_limit = 1000;

    let code = match input
    {
        YololInput::Yolol(code) => code,
        YololInput::CylonAst(_) => {
            return Err("Execution from Cylon AST not yet supported! Psst, try outputting yolol then using it as input ;)".to_owned())
        }
    };

    let lines: Vec<String> = code.lines().map(String::from).collect();
    let line_len: i64 = lines.len().try_into().unwrap();

    for _ in 0..tick_limit
    {
        // This is a stupid line but I can't find a better way to do it for some reason...
        let next_line = if env.next_line > line_len || env.next_line <= 0 { 1 } else { env.next_line };
        env.next_line = next_line;

        let next_line: usize = next_line.try_into().unwrap();

        yoloxide::execute_line(env, lines[next_line - 1].clone());
    }

    Ok(())
}

fn output_yolol(input: YololInput) -> Result<String, String>
{
    match parse_yolol(input)
    {
        Ok(prog) => Ok(format!("{}", prog)),
        Err(e) => Err(e)
    }
}

fn output_cylon_ast(input: YololInput) -> Result<String, String>
{
    let cylon_root = match input
    {
        YololInput::CylonAst(root) => root,

        yolol => match parse_yolol(yolol) {
            Ok(prog) => CylonRoot::new(prog.into()),
            Err(e) => return Err(e)
        }
    };

    match serde_json::to_string(&cylon_root)
    {
        Ok(ast) => Ok(ast),
        Err(error) => Err(format!("Converting AST to Cylon AST failed with error: ```{}```", error))
    }
}

fn output_ast(input: YololInput) -> Result<String, String>
{
    match parse_yolol(input)
    {
        Ok(prog) => Ok(format!("{:?}", prog)),
        Err(e) => Err(e)
    }
}

fn output_tokens(input: YololInput) -> Result<String, String>
{
    match tokenize_yolol(input)
    {
        Ok(tokens) => Ok(format!("{:?}", tokens)),
        Err(e) => Err(e)
    }
}

fn parse_yolol(input: YololInput) -> Result<Program, String>
{
    if let YololInput::CylonAst(root) = input
    {
        return Ok(root.program.try_into()?)
    }

    let tokens = tokenize_yolol(input)?;
    let mut window = VecWindow::new(tokens, 0);

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
    if input.contains('`')
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
            match output_execution(input, &mut env)
            {
                Ok(_) => (),
                Err(e) => {
                    message.channel_id.say(&context.http, e)?;
                    return Ok(())
                }
            }

            let output = env.to_string();
            if output.len() > 1900
            {
                use serenity::http::AttachmentType;
                let attachment = vec![AttachmentType::Bytes((output.as_bytes(), "toaster_output.txt"))];
                message.channel_id.send_files(&context.http, attachment, |m| m.content("The output was too long! Here's a file instead"))?;
            }
            else
            {
                let output = format!("Output environment from execution: ```{}```", output);
                message.channel_id.say(&context.http, output)?;
            }
        },
        OutputFlag::Yolol => {
            let output = match output_yolol(input)
            {
                Ok(o) => o,
                Err(e) => {
                    message.channel_id.say(&context.http, e)?;
                    return Ok(());
                } 
            };

            let output = format!("Reconstructed code: ```{}```", output);
            message.channel_id.say(&context.http, output)?;
        },
        OutputFlag::CylonAst => {
            let output = match output_cylon_ast(input)
            {
                Ok(o) => o,
                Err(e) => {
                    message.channel_id.say(&context.http, e)?;
                    return Ok(());
                }
            };

            use serenity::http::AttachmentType;

            if output.len() > 2000
            {
                let attachment = vec![AttachmentType::Bytes((output.as_bytes(), "cylon_ast.json"))];
                message.channel_id.send_files(&context.http, attachment, |m| m.content("The code was too long! Here's a file instead"))?;
            }
            else
            {
                let output = format!("Cylon AST of program:\n```json\n{}\n```", output);
                message.channel_id.say(&context.http, output)?;
            }

        },
        OutputFlag::Ast => {
            let output = match output_ast(input)
            {
                Ok(o) => o,
                Err(e) => {
                    message.channel_id.say(&context.http, e)?;
                    return Ok(());
                }
            };

            let output = format!("Parsed program: ```{:?}```", output);
            message.channel_id.say(&context.http, output)?;
        },
        OutputFlag::Tokens => {
            let output = match output_tokens(input)
            {
                Ok(o) => o,
                Err(e) => {
                    message.channel_id.say(&context.http, e)?;
                    return Ok(());
                }
            };

            let output = format!("Tokenized program: ```{:?}```", output);
            message.channel_id.say(&context.http, output)?;
        }
    }

    Ok(())
}