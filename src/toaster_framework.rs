#[macro_use]
mod utils;
mod groups;

use std::sync::{Arc, Mutex};
use std::mem;

use serenity::prelude::*;

use serenity::model::channel::Message;
use serenity::framework::{
    Framework,
    standard::{
        StandardFramework,
    }
};

use threadpool::ThreadPool;

static TOASTER_ID: u64 = 601092364181962762;

#[derive(Clone)]
pub struct ToasterFramework
{
    inner: Arc<Mutex<StandardFramework>>,
}

impl ToasterFramework
{
    
}

impl ToasterFramework
{
    pub fn new() -> ToasterFramework
    {
        let inner = Arc::new(Mutex::new(
            Self::new_inner("t>")
        ));

        ToasterFramework { inner }
    }

    pub fn new_inner(prefix: &str) -> StandardFramework
    {
        groups::framework_with_groups()
            .configure(|c| c
                .prefix(prefix)
                .on_mention(Some(TOASTER_ID.into()))
                .allow_dm(false))
    }

    pub fn replace_inner(&self, framework: StandardFramework)
    {
        mem::replace(&mut *self.inner.lock().unwrap(), framework);
    }
}

impl Framework for ToasterFramework
{
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool)
    {
        self.inner.lock().unwrap().dispatch(ctx, msg, threadpool);
    }
}

impl TypeMapKey for ToasterFramework
{
    type Value = ToasterFramework;
}