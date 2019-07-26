#[macro_use]
mod utils;

mod groups;

use serenity::framework::standard::{
    StandardFramework,
};

static TOASTER_ID: u64 = 601092364181962762;

pub struct ToasterFramework {}

impl ToasterFramework
{
    pub fn new() -> StandardFramework
    {
        groups::framework_with_groups()
            .configure(|c| c
                .prefix("t>")
                .on_mention(Some(TOASTER_ID.into()))
                .allow_dm(false))
    }
}



