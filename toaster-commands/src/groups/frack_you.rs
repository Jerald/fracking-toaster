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
    options: {
        prefix: "frack"
    },
    commands: [frack_you]
});

#[command("you")]
fn frack_you(context: &mut Context, message: &Message, _args: Args) -> CommandResult
{
    // match args.current()
    // {
    //     Some("you") | Some("you!") => { message.reply(context, "no, frack _you_")?; },
    //     _ => ()
    // }

    message.reply(context, "no, frack _you_")?;
    
    Ok(())
}