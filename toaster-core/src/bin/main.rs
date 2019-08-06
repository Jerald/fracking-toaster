use std::env;

use serenity::prelude::*;

use toaster_core::{
    toaster_framework::ToasterFramework,
    dynamic_loading::{
        PluginManager,
    },
    handler,
};

fn main()
{
    println!("Hello, world!");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Didn't find DISCORD_TOKEN env variable!"), handler::Handler)
        .expect("Error creating client!");

    let framework = {
        #[cfg(debug_assertions)]
        let plugin_manager = PluginManager::new("target/debug/libtoaster_commands.so", "/home/toaster/plugin_temp_dir");
        #[cfg(not(debug_assertions))]
        let plugin_manager = PluginManager::new("target/release/libtoaster_commands.so", "/home/toaster/plugin_temp_dir");

        ToasterFramework::new(plugin_manager.unwrap(), |c| c)
    };

    framework.add_all_groups().unwrap();

    {
        let mut data = client.data.write();
        data.insert::<ToasterFramework>(framework.clone());
    }

    client.with_framework(framework);

    if let Err(why) = client.start()
    {
        println!("An error occurred while starting the client: {:?}", why);
    }
}