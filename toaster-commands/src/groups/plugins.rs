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

use toaster_core::toaster_framework::ToasterFramework;

group!({
    name: "plugins",
    options: {},
    commands: [add_group, remove_group],
});

#[command]
fn add_group(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let group = match args.current()
    {
        Some(arg) => arg,
        None => {
            message.channel_id.say(&context.http, "No group supplied!")?;
            return Ok(())
        }
    };

    let data = context.data.read();
    let framework = data.get::<ToasterFramework>().unwrap();

    framework.flush_lib_buffer();

    match framework.add_group(group)
    {
        Err(error) => {
            message.channel_id.say(&context.http, format!("Failed to add group! Error: ```{}```", error))?;
            return Ok(());
        },
        _ => {}
    }

    message.channel_id.say(&context.http, format!("Added group `{}`!", group))?;
    Ok(())
}

#[command]
fn remove_group(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    let group = match args.current()
    {
        Some(arg) => arg,
        None => {
            message.channel_id.say(&context.http, "No group supplied!")?;
            return Ok(())
        }
    };

    let data = context.data.write();
    let framework = data.get::<ToasterFramework>().unwrap();

    framework.flush_lib_buffer();

    match framework.remove_group(group)
    {
        Err(error) => {
            message.channel_id.say(&context.http, format!("Failed to remove group! Error: ```{}```", error))?;
            return Ok(());
        },
        _ => {}
    }

    message.channel_id.say(&context.http, format!("Removed group `{}`!", group))?;
    Ok(())
}