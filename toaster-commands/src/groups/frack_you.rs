use serenity::prelude::*;
use serenity::model::channel::Message;

use serenity::framework::standard::{
    CommandResult,
    Args,
    macros::{
        command,
        group
    }
};

group!({
    name: "frack_you",
    options: {},
    commands: [frack]
});

#[command]
fn frack(context: &mut Context, message: &Message, args: Args) -> CommandResult
{
    match args.current()
    {
        Some("you") | Some("you!") => { message.reply(context, "no, frack _you_")?; },
        _ => ()
    }
    
    Ok(())
}