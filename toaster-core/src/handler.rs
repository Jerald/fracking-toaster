use serenity::prelude::*;

use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;

pub struct Handler;
impl EventHandler for Handler
{
    fn ready(&self, ctx: Context, _data_about_bot: Ready)
    {
        println!("Ready trigger fired!");
        let channel_id: ChannelId = {
            let json = std::fs::read_to_string("/home/toaster/fracking-toaster/.trigger").expect("Unable to read .trigger in startup!");
            if json.is_empty() { return }

            serde_json::from_str(&json).expect("Unable to serialize .trigger into a ChannelId!")
        };

        channel_id.say(&ctx.http, "I'm back online!").expect("Unable to report online status to startup channel!");
    }
}