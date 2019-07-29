use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    Args,
    CommandResult,
    macros::{
        command,
        group
    }
};

use crate::toaster_framework::{
    ToasterFramework
};

group!({
    name: "general",
    options: {},
    commands: [change_prefix, restart, test, ping, hello, help],
});

#[command]
fn change_prefix(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let prefix = match args.current()
    {
        Some(arg) => arg,
        None => {
            message.channel_id.say(&context.http, "No new prefix supplied! I need one for this to work...")?;
            return Ok(())
        }
    };

    let new_inner = ToasterFramework::new_inner(prefix);

    let data = context.data.read();
    let framework = data.get::<ToasterFramework>().unwrap();

    framework.replace_inner(new_inner);

    message.channel_id.say(&context.http, format!("My prefix has been changed to '{}'! Try it out", prefix))?;
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