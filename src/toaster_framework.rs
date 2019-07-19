use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    StandardFramework,
    CommandGroup,
    CommandResult,
    Args,
    macros::{
        command,
        group
    }
};

pub struct ToasterFramework {}

impl ToasterFramework
{
    pub fn new() -> StandardFramework
    {
        StandardFramework::new()
            .configure(|c| c
                .prefix("t>")
                .allow_dm(false))
            .group(&GENERAL_GROUP)
    }
}

group!({
    name: "general",
    options: {},
    commands: [yolol, restart, test, ping, hello, help],
});

#[command]
fn yolol(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let mut flag_tokenize = false;
    let mut flag_parse = false;
    let mut flag_code = false;

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
            _ => break
        }
    }

    let code_arg = args.rest();
    println!("Attempt to stringify: {:?}", code_arg);

    let code = {
        use regex::Regex;
        let code_matcher = Regex::new(r"\A(?s:\n*)```\n?((?s).*)\n?```\z").expect("Code matching regex failed to compile!");

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

    if flag_tokenize || flag_parse || flag_code
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
            for line in parsed_program
            {
                output += &format!("{}\n", &line);
            }
            format!("Reconstructed code:\n```{}```", output)
        }
        else
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

#[command]
fn restart(context: &mut Context, message: &Message) -> CommandResult
{
    {
        let channel_json = serde_json::to_string(&message.channel_id)?;

        use std::fs;
        use std::io::Write;

        let mut trigger_file = fs::File::create("/home/toaster/fracking-toaster/.trigger")?;
        trigger_file.write_all(channel_json.as_bytes())?;
    }

    message.channel_id.say(&context.http, "Touching trigger to restart bot...")?;

    use std::process::Command as RustCommand;
    RustCommand::new("touch")
        .arg("/home/toaster/fracking-toaster/.trigger")
        .output()
        .expect("Failed to touch trigger!");

    Ok(())
}

#[command]
fn test(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.channel_id.say(&ctx.http, "I've been refactored, _slightly_")?;
    Ok(())
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.reply(ctx, "Pong! Was I dead?")?;
    Ok(())
}

#[command]
fn hello(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.channel_id.say(&ctx.http, "Hi!")?;
    Ok(())
}

#[command]
fn help(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.channel_id.say(&ctx.http, "I'm just a toaster! What would I be able to do to help?\n (Psst: I'm under construction, check back in later!)")?;
    Ok(())
}