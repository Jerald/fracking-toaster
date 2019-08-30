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

use toaster_core::{
    share_map_hack::ToasterHack,
};

group!({
    name: "plugins",
    options: {
        default_command: list_groups,
        prefixes: ["plugin", "plugins"],
        allowed_roles: ["Bot Admin"],
    },
    commands: [add_group, remove_group, list_groups, reload_group, flush_buffer],
});

#[command("add")]
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

    let framework = {
        let data = context.data.read();
        data.get_toaster().expect("No ToasterFramework in data map!")
    };

    // Flushing the library buffer when the plugins module is in it will do _bad_ things.
    // Hopefully this will stop those bad things...
    if group == "plugins"
    {
        message.channel_id.say(&context, "You're trying to add the plugins group for some reason! I'm assuming this means it was recently removed, so let's skip flushing the library buffer so things don't break. You're welcome!")?;
    }
    else
    {
        framework.flush_lib_buffer();
    }

    if let Err(error) = framework.add_group(group)
    {
        message.channel_id.say(&context.http, format!("Failed to add group! Error: ```{}```", error))?;
        return Ok(());
    }

    message.channel_id.say(&context.http, format!("Added group `{}`!", group))?;
    Ok(())
}

#[command("remove")]
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

    let framework = {
        let data = context.data.read();
        data.get_toaster().expect("No ToasterFramework in data map!")
    };

    framework.flush_lib_buffer();

    if let Err(error) = framework.remove_group(group)
    {
        message.channel_id.say(&context.http, format!("Failed to remove group! Error: ```{}```", error))?;
        return Ok(());
    }

    message.channel_id.say(&context.http, format!("Removed group `{}`!", group))?;
    Ok(())
}

#[command("list")]
fn list_groups(context: &mut Context, message: &Message) -> CommandResult
{
    let groups = {
        let data = context.data.read();
        let framework = data.get_toaster().expect("ToasterFramework should be in my data map...");

        framework.flush_lib_buffer();
        framework.get_group_list()
    };

    let mut output_string = String::from("```Groups found:\n\n");
    for group in groups
    {
        output_string += &format!("- {}\n", group);
    }
    output_string += "```";

    message.channel_id.say(&context.http, output_string)?;
    Ok(())
}

#[command("reload")]
fn reload_group(context: &mut Context, message: &Message, mut args: Args) -> CommandResult
{
    let group = match args.current()
    {
        // Some(arg) if arg == "--all" => {
        //     let data = context.data.read();
        //     let framework = data.get_toaster().expect("ToasterFramework should be in my data map...");

        //     for group in &framework.get_group_list()
        //     {
        //         if let Err(error) = framework.remove_group(group)
        //         {
        //             message.channel_id.say(&context.http, format!("Failed to remove group! Error: ```{}```", error))?;
        //             return Ok(());
        //         }
        //     }
        //     // framework
        // },

        Some(arg) => arg,

        None => {
            message.channel_id.say(&context.http, "No group supplied!")?;
            return Ok(())
        }
    };

    // If the user tried to reload the plugins groups, require confirmation.
    // Reloading this group can have some bad side effects...
    if group == "plugins"
    {
        match args.advance().current()
        {
            Some(arg) if arg == "--confirm" => { args.rewind(); },
            _ => {
                message.channel_id.say(&context, "Reloading the plugins group is scary and can break things mysteriously! To be sure you want to do this, run the command with the `--confirm` flag.")?;
                return Ok(());  
            }
        }
    }

    remove_group(context, message, args.clone())?;

    {
        let build_msg = message.channel_id.say(&context, "Running cargo build to ensure plugin is up to date...")?;

        use super::cargo;
        cargo::cargo_build(context, message, args.clone())?;

        build_msg.delete(&context)?;
    }

    add_group(context, message, args)?;

    Ok(())
}

#[command("flush")]
fn flush_buffer(context: &mut Context, message: &Message) -> CommandResult
{
    let start_msg = message.channel_id.say(&context, "Flushing plugin buffer...")?;

    {
        let data = context.data.read();
        let framework = data.get_toaster().expect("ToasterFramework should be in my data map...");

        framework.flush_lib_buffer();
    }

    start_msg.delete(&context)?;
    message.channel_id.say(&context, "Flushed plugin buffer!")?;

    Ok(())
}