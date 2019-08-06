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

group!({
    name: "general",
    options: {},
    commands: [restart, test, ping, hello, help, franken_toaster],
});

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
    msg.channel_id.say(&ctx.http, "I think I'm working...")?;
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

#[command]
fn franken_toaster(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.channel_id.say(&ctx.http, "Not sure if I'm alive")?;
    Ok(())
}