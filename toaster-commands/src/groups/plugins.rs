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
    options: {
        default_command: list_groups,
        prefix: "plugin",
        allowed_roles: ["Bot Admin"],
    },
    commands: [add_group, remove_group, list_groups],
});

#[command]
#[aliases("add")]
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

    if let Err(error) = framework.add_group(group)
    {
        message.channel_id.say(&context.http, format!("Failed to add group! Error: ```{}```", error))?;
        return Ok(());
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

    let data = context.data.read();
    let framework = data.get::<ToasterFramework>().unwrap();

    framework.flush_lib_buffer();

    if let Err(error) = framework.remove_group(group)
    {
        message.channel_id.say(&context.http, format!("Failed to remove group! Error: ```{}```", error))?;
        return Ok(());
    }

    message.channel_id.say(&context.http, format!("Removed group `{}`!", group))?;
    Ok(())
}

#[command]
fn list_groups(context: &mut Context, message: &Message) -> CommandResult
{
    let data = context.data.read();
    let framework = data.get::<ToasterFramework>().expect("ToasterFramework should definitely be in my data map...");

    framework.flush_lib_buffer();

    let groups = framework.get_group_list();

    let mut output_string = String::from("```Groups found:\n");
    for group in groups
    {
        output_string += &format!("- {}\n", group);
    }
    output_string += "```";

    message.channel_id.say(&context.http, output_string)?;
    Ok(())
}