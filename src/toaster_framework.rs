#[macro_use]
mod utils;

mod groups;

use serenity::framework::standard::{
    StandardFramework,
};

pub struct ToasterFramework {}

impl ToasterFramework
{
    pub fn new() -> StandardFramework
    {
        groups::framework_with_groups()
            .configure(|c| c
                .prefix("t>")
                .allow_dm(false))
    }
}



