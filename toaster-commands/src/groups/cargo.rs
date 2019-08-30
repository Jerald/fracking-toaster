use std::process::{
    Command as RustCommand,
    Stdio,
};

use std::io::Read;

use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    CommandResult,
    CommandError,
    Args,
    macros::{
        command,
        group
    }
};

group!({
    name: "cargo",
    options: {
        prefix: "cargo"
    },
    commands: [cargo_build, cargo_check, cargo_touch]
});

// TODO: make this do things...
// fn command_executor_async() -> CommandResult
// {
//     let msg_edit = |stream: &str, content: &str| format!("{} from executing cargo command: ```\n{}\n```", stream, content);

//     // let mut stdout = String::new();
//     let mut stdout_msg = message.reply(&context, "Stdout from executing cargo command: ```\n\n```")?;

//     // let mut stderr = String::new();
//     let mut stderr_msg = message.reply(&context, "Stderr from executing cargo command: ```\n\n```")?;

//     stdout_msg.edit(&context, |m| m.content(msg_edit("Stdout", &stdout)));
// }

fn command_executor(context: &mut Context, message: &Message, command: &mut RustCommand) -> CommandResult
{
    let start_msg = message.channel_id.say(&context, "Starting cargo command...")?;

    let output = command.output();

    match output
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            if !stdout.is_empty()
            {
                message.channel_id.say(&context, format!("Stdout from executing cargo command: ```\n{}\n```", stdout))?;
            }

            if !stderr.is_empty()
            {
                message.channel_id.say(&context, format!("Stderr from executing cargo command: ```\n{}\n```", stderr))?;
            }
        },
        Err(e) => {
            message.reply(&context, format!("Error executing cargo build! Error ```\n{}\n```", e))?;
        }
    }

    start_msg.delete(&context)?;

    Ok(())
}

#[command("build")]
pub fn cargo_build(context: &mut Context, message: &Message) -> CommandResult
{
    command_executor(context, message,
        RustCommand::new("cargo")
            .arg("build")
            .arg("--release"))
}

#[command("check")]
fn cargo_check(context: &mut Context, message: &Message) -> CommandResult
{
    command_executor(context, message,
        RustCommand::new("cargo")
            .arg("check"))
}

#[command("touch")]
fn cargo_touch(context: &mut Context, message: &Message) -> CommandResult
{
    command_executor(context, message,
        RustCommand::new("touch")
            .arg("README.md"))
}