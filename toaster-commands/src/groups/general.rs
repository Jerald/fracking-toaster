use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::{Deref, DerefMut, Drop};

use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    Args,
    CommandResult,
    StandardFramework,
    macros::{
        command,
        group
    }
};

use libloading::{
    Library,
    Symbol,
    Result
};

use toaster_core::{
    toaster_framework::ToasterFramework,
    dynamic_loading::{CommandLib},
};

group!({
    name: "general",
    options: {},
    commands: [reload_commands, add_group, remove_group, restart, test, ping, hello, help, franken_toaster],
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

    {
        let mut data = context.data.write();
        let framework = data.get_mut::<ToasterFramework>().unwrap();

        framework.add_group(group)?;
    }

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

    {
        let mut data = context.data.write();
        let framework = data.get_mut::<ToasterFramework>().unwrap();

        framework.remove_group(group)?;
    }

    Ok(())
}

#[command]
fn reload_commands(context: &mut Context, message: &Message, _args: Args) -> CommandResult
{
    println!("Starting reload_commands...");

    let mut data = context.data.write();

    // Reload the dynamic library into a wrapper
    let command_lib = {
        println!("Loading new lib...");
        let lib = Library::new("target/debug/libtoaster_commands.so").expect("Failed to open shared library in reload_commands!");
        CommandLib(Arc::new(lib))
    };

    // Replace the wrapper in the data map with the one from our new library
    // We return the old library so it's not dropped until this command finishes execution
    let _previous_lib = {
        println!("Getting previous CommandLib...");
        data.insert::<CommandLib>(command_lib.clone()).unwrap();
    };

    // // Loads the framework factory symbol and uses it to construct a new framework inner
    // let new_inner = {
    //     println!("Extracting new factory symbol...");
    //     let factory: Symbol<fn() -> StandardFramework> = unsafe { command_lib.0.get(b"framework_factory\0").expect("Failed to unwrap framework_factory symbol in reload_commands") };

    //     println!("Creating new inner for framework...");
    //     toaster_core::toaster_framework::ToasterFramework::create_inner(*factory, "t>")
    // };

    // // Grabs the framework out of the data map and swaps the inner with the newly constructed one
    // {
    //     let framework = data.get::<ToasterFramework>().unwrap();

    //     println!("Swapping inners...");
    //     let mut old_inner = framework.get_writer().unwrap();

    //     *old_inner = new_inner;
    // }

    println!("Finished reloading commands!");
    message.channel_id.say(&context.http, "Finished reloading commands!")?;

    Ok(())
}

// #[command]
// fn change_prefix(context: &mut Context, message: &Message, args: Args) -> CommandResult
// {
//     let prefix = match args.current()
//     {
//         Some(arg) => arg,
//         None => {
//             message.channel_id.say(&context.http, "No new prefix supplied! I need one for this to work...")?;
//             return Ok(())
//         }
//     };

//     let data = context.data.read();
//     let framework = data.get::<ToasterFramework>().unwrap();

//     let guard = data.get::<PluginManager>().unwrap().command_lib.lock().unwrap();

//     println!("Re-getting framework_factory symbol...");
//     let factory: Symbol<unsafe extern fn() -> StandardFramework> = unsafe { guard.get(b"framework_factory\0").expect("Failed to unwrap framework_factory symbol in main.rs!") };

//     println!("Attempting to re-gotten factory symbol...");
//     let mut new_inner = unsafe { ToasterFramework::unsafe_create_inner(*factory, prefix) };

//     // let mut new_inner = ToasterFramework::create_inner(toaster_utils::toaster_framework::default_inner_factory, prefix);
//     println!("Swapping new inner with old inner...");
//     framework.swap_inner(&mut new_inner);

//     message.channel_id.say(&context.http, format!("My prefix has been changed to '{}'! Try it out", prefix))?;
//     Ok(())
// }

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
    msg.channel_id.say(&ctx.http, "I might be alive...")?;
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