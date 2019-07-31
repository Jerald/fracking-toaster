use std::sync::{Arc, Mutex};

use libloading::{
    Library,
    Symbol,
    Result
};

use serenity::prelude::TypeMapKey;
use serenity::framework::{
    Framework,
    standard::{
        StandardFramework,
    }
};

use crate::{
    toaster_framework,
    toaster_framework::ToasterFramework,
    handler,
};

#[derive(Clone)]
pub struct CommandLib(pub Arc<Library>);

impl TypeMapKey for CommandLib
{
    type Value = CommandLib;
}