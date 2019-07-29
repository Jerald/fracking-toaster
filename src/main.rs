// Used for group construction macro
#![feature(proc_macro_hygiene)]

use std::env;

use serenity::prelude::*;

mod handler;
use handler::Handler;

mod toaster_framework;
use toaster_framework::ToasterFramework;


fn main()
{
    println!("Hello, world!");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Didn't find DISCORD_TOKEN env variable!"), Handler)
        .expect("Error creating client!");

    let framework = ToasterFramework::new();

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