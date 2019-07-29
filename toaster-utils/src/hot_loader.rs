use std::sync::{Arc, Mutex, MutexGuard};

use dynamic_reload::{
    DynamicReload,
    Lib,
    Symbol,
    Search,
    PlatformName,
    UpdateState
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

pub struct PluginManager
{
    pub reloader: DynamicReload<'static>,
}

impl TypeMapKey for PluginManager
{
    type Value = Arc<Mutex<PluginManager>>;
}

impl PluginManager
{
    pub fn reload_callback(guard: &mut MutexGuard<StandardFramework>, state: UpdateState, lib: Option<&Arc<Lib>>)
    {
        match state
        {
            UpdateState::Before => {},
            UpdateState::After => {
                let lib = &lib.unwrap().lib;
                let factory: Symbol<unsafe extern fn() -> StandardFramework> = unsafe { lib.get(b"framework_factory\0").unwrap() };
                
                let mut framework = toaster_framework::ToasterFramework::unsafe_create_inner(*factory, "t>");
                toaster_framework::ToasterFramework::swap_with_lock(guard, &mut framework);
            },
            UpdateState::ReloadFailed(error) => println!("Failed to reload! Error: {}", error)
        }
    }
}

// pub struct HotReloadWrapper
// {
//     guard: Option<MutexGuard<'_, StandardFramework>>,
//     old_inner: Option<StandardFramework>,
//     error: Option<String>
//     // Pre(ToasterFramework),
//     // Before(MutexGuard<'a, StandardFramework>),
//     // After(StandardFramework),
//     // Error(String)
// }

// impl Default for HotReloadWrapper
// {
//     fn default() -> HotReloadWrapper
//     {
//         HotReloadWrapper {
//             framework: None,
//             guard: None,
//             old_inner: None,
//             error: None
//         }
//     }
// }