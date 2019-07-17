use std::env;

use serenity::prelude::*;

use serenity::model::channel::Message;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

group!({
    name: "general",
    options: {},
    commands: [ping],
});

struct Handler;
impl EventHandler for Handler {}

fn main() {
    println!("Hello, world!");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Didn't find DISCORD_TOKEN env variable!"), Handler)
        .expect("Error creating client!");

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(">"))
        .group(&GENERAL_GROUP));

    if let Err(why) = client.start()
    {
        println!("An error occured while starting the client: {:?}", why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult
{
    msg.reply(ctx, "Pong!")?;
    Ok(())
}
