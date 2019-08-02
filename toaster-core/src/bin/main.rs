// Used for group construction macro
#![feature(proc_macro_hygiene)]

use std::env;

use std::sync::{Arc, Mutex};

use serenity::prelude::*;
use serenity::framework::{
    StandardFramework,
    standard::CommandGroup,
};

use libloading::{
    Library,
    Symbol,
    Result
};

use toaster_core::{
    toaster_framework,
    toaster_framework::ToasterFramework,
    dynamic_loading::{CommandLib},
    handler,
};

fn main()
{
    println!("Hello, world!");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Didn't find DISCORD_TOKEN env variable!"), handler::Handler)
        .expect("Error creating client!");

    let command_lib = {
        println!("Attempting to open shared library for initial loading...");
        let lib = Library::new("target/debug/libtoaster_commands.so").expect("Failed to open shared library in main.rs!");

        CommandLib(Arc::new(lib))
    };

    let groups_slice = {
        let get_slice: Symbol<fn() -> &'static [&'static CommandGroup]> = unsafe { command_lib.0.get(b"get_group_slice\0").expect("Failed to unwrap get_group_slice symbol in main.rs!") };
        (*get_slice)()
    };

    let framework = {
        // println!("Getting initial framework_factory symbol...");
        // let factory: Symbol<fn() -> StandardFramework> = unsafe { command_lib.0.get(b"framework_factory\0").expect("Failed to unwrap framework_factory symbol in main.rs!") };

        // println!("Attempting to use initial factory symbol...");
        Arc::new(ToasterFramework::new(None, |c| c))
    };

    {
        let mut data = client.data.write();
        data.insert::<ToasterFramework>(Arc::clone(&framework));
        data.insert::<CommandLib>(command_lib);
    }

    client.with_framework(framework);

    if let Err(why) = client.start()
    {
        println!("An error occurred while starting the client: {:?}", why);
    }
}