// Used for group construction macro
#![feature(proc_macro_hygiene)]

use std::env;

use std::sync::{Arc, Mutex};

use serenity::prelude::*;

use dynamic_reload::{
    DynamicReload,
    Lib,
    Symbol,
    Search,
    PlatformName,
    UpdateState
};

use toaster_utils::{
    toaster_framework::ToasterFramework,
    hot_loader::{PluginManager},
    handler,
};

fn main()
{
    println!("Hello, world!");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Didn't find DISCORD_TOKEN env variable!"), handler::Handler)
        .expect("Error creating client!");

    let framework = ToasterFramework::new(toaster_commands::framework_factory);
    let plugin_manager = {
        let reloader = DynamicReload::new(Some(vec!["target/debug"]), Some("target/debug"), Search::Default);
        PluginManager {
            reloader
        }
    };

    let wrapped_plugin_manager = Arc::new(Mutex::new(plugin_manager));

    {
        let mut data = client.data.write();
        data.insert::<ToasterFramework>(framework.clone());
        data.insert::<PluginManager>(wrapped_plugin_manager.clone());
    }

    client.with_framework(framework);

    if let Err(why) = client.start()
    {
        println!("An error occurred while starting the client: {:?}", why);
    }
}