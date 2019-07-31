use std::sync::{Arc, Mutex, LockResult, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
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

type InnerFactory = fn() -> StandardFramework;
type UnsafeInnerFactory = unsafe fn() -> StandardFramework;

#[derive(Clone)]
pub struct ToasterFramework
{
    pub framework: Arc<RwLock<StandardFramework>>,
}

pub fn default_inner_factory() -> StandardFramework
{
    StandardFramework::new()
}

impl ToasterFramework
{
    pub fn new(factory: Option<InnerFactory>) -> ToasterFramework
    {
        let inner_factory = match factory
        {
            Some(factory) => factory,
            None => default_inner_factory
        };

        let framework = Arc::new(RwLock::new(
            Self::create_inner(inner_factory, "t>")
        ));

        ToasterFramework {
            framework,
        }
    }

    pub fn create_inner(inner_factory: InnerFactory, prefix: &str) -> StandardFramework
    {
        inner_factory()
            .configure(|c| c
                .prefix(prefix)
                .on_mention(Some(TOASTER_ID.into()))
                .allow_dm(false))
    }

    pub fn swap_inner(&self, new: &mut StandardFramework)
    {
        mem::swap(&mut *self.framework.write().unwrap(), new);
    }

    pub fn get_reader(&self) -> LockResult<RwLockReadGuard<StandardFramework>>
    {
        self.framework.read()
    } 

    pub fn get_writer(&self) -> LockResult<RwLockWriteGuard<StandardFramework>>
    {
        self.framework.write()
    }
}

impl Framework for ToasterFramework
{
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool)
    {
        self.framework.write().unwrap().dispatch(ctx, msg, threadpool);
    }
}

impl TypeMapKey for ToasterFramework
{
    type Value = ToasterFramework;
}