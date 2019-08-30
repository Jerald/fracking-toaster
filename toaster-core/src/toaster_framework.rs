use std::sync::{
    Arc,
    Weak,
    // Mutex, MutexGuard
};

use parking_lot::{
    Mutex, MutexGuard
};

use serenity::prelude::*;

use serenity::model::channel::Message;
use serenity::framework::{
    Framework,
    standard::{
        StandardFramework,
        Configuration,
    }
};

use crate::dynamic_loading::{
    GroupLib,
    PluginManager
};

use threadpool::ThreadPool;

static TOASTER_ID: u64 = 601092364181962762;
static TOASTER_PREFIX: &str = "t>";

type RawInnerFactory = fn() -> StandardFramework;
pub fn default_raw_inner_factory() -> StandardFramework
{
    StandardFramework::new()
}

pub struct ToasterFramework
{
    inner: Arc<Mutex<StandardFramework>>,
    plugin_manager: Arc<PluginManager>
}

// Ensures it clones correctly
impl Clone for ToasterFramework
{
    fn clone(&self) -> Self
    {
        ToasterFramework {
            inner: Arc::clone(&self.inner),
            plugin_manager: Arc::clone(&self.plugin_manager)
        }
    }
}

impl TypeMapKey for ToasterFramework
{
    type Value = ToasterFramework;
}

type ConfigFn = fn(&mut Configuration) -> &mut Configuration;

impl ToasterFramework
{
    const DEFAULT_CONFIG: ConfigFn = |conf| { conf
        .prefix(TOASTER_PREFIX)
        .on_mention(Some(TOASTER_ID.into()))
        .with_whitespace(true)
        .allow_dm(false)
    };

    pub fn new<F>(plugin_manager: PluginManager, config: F) -> ToasterFramework
        where F: FnOnce(&mut Configuration) -> &mut Configuration
    {
        let inner = Arc::new(Mutex::new(
            Self::create_raw_inner(default_raw_inner_factory)
                .configure(config)
        ));

        let plugin_manager = Arc::new(plugin_manager);

        ToasterFramework {
            inner,
            plugin_manager,
        }
    }

    pub fn add_all_groups(&self) -> Result<(), String>
    {
        // Lock mutex now since the entire process should be protected
        let mut lock = self.inner.lock();

        let group_lib_vec = self.plugin_manager.load_all_groups()?;

        for group_lib in group_lib_vec
        {
            self.add_group_impl(group_lib, &mut lock)?;
        }

        Ok(())
    }

    pub fn add_group(&self, group: &str) -> Result<(), String>
    {
        // Lock mutex now since the entire process should be protected
        let mut lock = self.inner.lock();

        let group_lib = self.plugin_manager.load_group(group)?;
        self.add_group_impl(group_lib, &mut lock)
    }

    fn add_group_impl(&self, group_lib: Weak<GroupLib>, lock: &mut MutexGuard<StandardFramework>) -> Result<(), String>
    {
        let group = match group_lib.upgrade()
        {
            Some(group_lib) => group_lib.group,
            None => return Err("[ToasterFramework::add_group] weak pointer from load_group has expired!".to_owned())
        };

        println!("[ToasterFramework::add_group_impl] Adding group: '{}'", group.name);
        lock.group_add(group);

        Ok(())
    }

    pub fn remove_group(&self, group: &str) -> Result<(), String>
    {
        // Lock mutex now since the entire process should be protected
        let mut lock = self.inner.lock();

        let group_lib = self.plugin_manager.unload_group(group);

        let group = match group_lib
        {
            Some(group_lib) => group_lib.group,
            None => return Err("[ToasterFramework::remove_group] tried to remove a group that wasn't loaded!".to_owned())
        };

        // Normally it might make more sense to remove the group from being usable
        // before we unload it. Luckily, the only "fail" case for unloading is if
        // the group was never loaded in the first place. That should make it safe.
        println!("[ToasterFramework::remove_group] Removing group: '{}'", group.name);
        lock.group_remove(group);

        Ok(())
    }

    pub fn flush_lib_buffer(&self)
    {
        self.plugin_manager.flush_unload_buffer();
    }

    pub fn get_group_list(&self) -> Vec<String>
    {
        self.plugin_manager.list_groups()
    }

    pub fn create_raw_inner(raw_inner_factory: RawInnerFactory) -> StandardFramework
    {
        raw_inner_factory()
            .configure(Self::DEFAULT_CONFIG)
    }

    // pub fn get_inner(&self) -> MutexGuard<StandardFramework>
    // {
    //     self.inner.lock()
    // } 
}

impl Framework for ToasterFramework
{
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool)
    {
        // let mut lock = self.inner.lock().expect("[ToasterFramework::dispatch] Poisoned mutex!");
        let mut lock = self.inner.lock();
        lock.dispatch(ctx, msg, threadpool);
    }
}