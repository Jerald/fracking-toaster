use std::sync::{Arc, Mutex, LockResult, MutexGuard};
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

#[derive(Clone)]
pub struct ToasterFramework
{
    inner_factory: InnerFactory,
    framework: Arc<Mutex<StandardFramework>>,
}

impl ToasterFramework
{
    pub fn new(inner_factory: InnerFactory) -> ToasterFramework
    {
        let framework = Arc::new(Mutex::new(
            Self::create_inner(inner_factory, "t>")
        ));

        ToasterFramework {
            inner_factory,
            framework,
        }
    }

    pub fn new_inner(&self, prefix: &str) -> StandardFramework
    {
        Self::create_inner(self.inner_factory, prefix)
    }

    fn create_inner(inner_factory: InnerFactory, prefix: &str) -> StandardFramework
    {
        inner_factory()
            .configure(|c| c
                .prefix(prefix)
                .on_mention(Some(TOASTER_ID.into()))
                .allow_dm(false))
    }

    pub fn unsafe_create_inner(inner_factory: unsafe extern fn() -> StandardFramework, prefix: &str) -> StandardFramework
    {
        unsafe { inner_factory() }
            .configure(|c| c
                .prefix(prefix)
                .on_mention(Some(TOASTER_ID.into()))
                .allow_dm(false))
    }

    pub fn swap_inner(&self, new: &mut StandardFramework)
    {
        mem::swap(&mut *self.framework.lock().unwrap(), new);
    }

    pub fn swap_with_lock(lock: &mut MutexGuard<StandardFramework>, new: &mut StandardFramework)
    {
        use std::ops::DerefMut;
        mem::swap(lock.deref_mut(), new)
    }

    pub fn get_inner(&self) -> LockResult<MutexGuard<StandardFramework>>
    {
        self.framework.lock()
    }
}

impl Framework for ToasterFramework
{
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool)
    {
        self.framework.lock().unwrap().dispatch(ctx, msg, threadpool);
    }
}

impl TypeMapKey for ToasterFramework
{
    type Value = ToasterFramework;
}