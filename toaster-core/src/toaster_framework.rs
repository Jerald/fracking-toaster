// use std::sync::{Arc, Mutex, LockResult, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::sync::{
    Arc,
    Weak
};

use std::cell::UnsafeCell;

use parking_lot::{
    Mutex,
    MutexGuard
};

use std::mem;

use std::collections::HashMap;

use serenity::prelude::*;

use serenity::model::channel::Message;
use serenity::framework::{
    Framework,
    standard::{
        StandardFramework,
        Configuration,
        CommandGroup,
        GroupOptions,
    }
};

use threadpool::ThreadPool;

static TOASTER_ID: u64 = 601092364181962762;
static TOASTER_PREFIX: &str = "t>";

static PARENT_GROUP_OPTIONS: GroupOptions = GroupOptions {
    prefixes: &[],
    allowed_roles: &[],
    required_permissions: serenity::model::permissions::Permissions { bits: 0u64 },
    owner_privilege: true,
    owners_only: false,
    help_available: true,
    only_in: serenity::framework::standard::OnlyIn::None,
    description: None,
    checks: &[],
    default_command: None,
};

static EMPTY_GROUP: CommandGroup = CommandGroup {
    help_name: "empty",
    name: "empty",
    options: &PARENT_GROUP_OPTIONS,
    commands: &[],
    sub_groups: &[],
};

static TEST_GROUP: CommandGroup = CommandGroup {
    help_name: "TEST",
    name: "TEST",
    options: &PARENT_GROUP_OPTIONS,
    commands: &[],
    sub_groups: &[],
};

// static mut SUB_GROUPS_INNER: [&'static CommandGroup; SubGroupWrapper::SUB_GROUPS_LEN] = [&EMPTY_GROUP; SubGroupWrapper::SUB_GROUPS_LEN];

// pub static SUB_GROUP_WRAPPER: SubGroupWrapper = {
//     let raw_inner = SUB_GROUPS_INNER;

//     SubGroupWrapper {
//         inner: Mutex::new(raw_inner),
//         raw_ref: unsafe { &SUB_GROUPS_INNER }
//     }
// };

// pub struct SubGroupWrapper
// {
//     inner: Mutex<[&'static CommandGroup; Self::SUB_GROUPS_LEN]>,
//     raw_ref: &'static [&'static CommandGroup]
// }

// impl SubGroupWrapper
// {
//     const SUB_GROUPS_LEN: usize = 10;

//     pub const unsafe fn get_raw_ref(&self) -> &'static [&'static CommandGroup]
//     {
//         self.raw_ref
//     }

//     pub unsafe fn subgroup_test()
//     {
//         SUB_GROUPS_INNER[0] = &TEST_GROUP;
//     }

//     pub fn add_subgroup(&self, new: &'static CommandGroup)
//     {
//         // Hold the lock to uphold our unsafe invarient: single-threaded access
//         let mut locked_inner = self.inner.lock();

//         // Find the first empty group reference in the array and replace it with the provided group
//         for group in locked_inner.iter_mut()
//         {
//             if *group == &EMPTY_GROUP
//             {
//                 std::mem::replace(group, new);
//                 break;
//             }
//         }
//     }

//     pub fn remove_subgroup(&self, to_remove: &'static CommandGroup)
//     {
//         // Hold the lock to uphold our unsafe invarient: single-threaded access
//         let mut locked_inner = self.inner.lock();

//         // Find _all_ references to this sub-group and remove them from the array
//         for group in locked_inner.iter_mut()
//         {
//             if *group == to_remove
//             {
//                 std::mem::replace(group, &EMPTY_GROUP);
//             }
//         }
//     }

//     pub fn get_subgroup(&self, index: usize) -> &'static CommandGroup
//     {
//         let mut locked_inner = self.inner.lock();

//         locked_inner[index]
//     }
// }

// Wrap this garbage in some static struct with consts in the impl
// By having _that_ behind a mutex, Sync behaviour will be maintained and the lock logic will be nicer
// const EMPTY_SUB_GROUP_LEN: usize = 10;
// static mut EMPTY_SUB_GROUPS: [&'static CommandGroup; EMPTY_SUB_GROUP_LEN] = [&EMPTY_GROUP; EMPTY_SUB_GROUP_LEN];

static PARENT_GROUP: CommandGroup = CommandGroup {
    help_name: "parent",
    name: "parent",
    options: &PARENT_GROUP_OPTIONS,
    commands: &[],
    sub_groups: &[],
};

type RawInnerFactory = fn() -> StandardFramework;
pub fn default_raw_inner_factory() -> StandardFramework
{
    StandardFramework::new()
}

pub struct ToasterFramework
{
    inner: Mutex<StandardFramework>,
    group_map: HashMap<String, &'static CommandGroup>
}

impl TypeMapKey for ToasterFramework
{
    type Value = Arc<ToasterFramework>;
}

impl Default for ToasterFramework
{
    fn default() -> Self
    {
        let inner = Mutex::new(
            Self::create_raw_inner(default_raw_inner_factory)
        );

        let group_map = HashMap::new();

        ToasterFramework {
            inner,
            group_map
        }
    }
}

type ConfigFn = fn(&mut Configuration) -> &mut Configuration;

impl ToasterFramework
{
    const DEFAULT_CONFIG: ConfigFn = { |conf| conf
        .prefix(TOASTER_PREFIX)
        .on_mention(Some(TOASTER_ID.into()))
        .allow_dm(false)
    };

    pub fn new<F>(raw_inner_factory: Option<RawInnerFactory>, config: F) -> ToasterFramework
        where F: FnOnce(&mut Configuration) -> &mut Configuration
    {
        let raw_inner_factory = match raw_inner_factory
        {
            Some(f) => f,
            None => return Self::default()
        };

        let inner = Mutex::new(
            Self::create_raw_inner(raw_inner_factory)
                .configure(config)
        );

        ToasterFramework {
            inner,
            ..Self::default()
        }
    }

    pub fn add_group(&self, group: &str) -> Result<(), String>
    {
        let group = match self.group_map.get(group)
        {
            Some(group) => group,
            None => return Err("Attempted to add a group that's not in the group_map!".to_owned())
        };

        let mut lock = self.inner.lock();
        lock.group_add(group);

        Ok(())
    }

    pub fn remove_group(&self, group: &str) -> Result<(), String>
    {
        let group = match self.group_map.get(group)
        {
            Some(group) => group,
            None => return Err("Attempted to remove group that's not in the map!".to_owned())
        };

        let mut lock = self.inner.lock();
        lock.group_remove(group);

        Ok(())
    }

    pub fn load_from_slice(&mut self, slice: &[&'static CommandGroup]) -> Result<(), String>
    {
        for group in slice
        {
            println!("Loading group: {}", group.name);
            self.group_map.insert(String::from(group.name), group);
            self.add_group(group.name)?;
        }

        Ok(())
    }

    pub fn create_raw_inner(raw_inner_factory: RawInnerFactory) -> StandardFramework
    {
        raw_inner_factory()
            .configure(Self::DEFAULT_CONFIG)
    }

    pub fn swap_inner(&self, new: &mut StandardFramework)
    {
        mem::swap(&mut *self.inner.lock(), new);
    }

    pub fn get_inner(&self) -> MutexGuard<StandardFramework>
    {
        self.inner.lock()
    } 
}

impl Framework for ToasterFramework
{
    #[inline]
    fn dispatch(&mut self, ctx: Context, msg: Message, threadpool: &ThreadPool)
    {
        self.inner.lock().dispatch(ctx, msg, threadpool);
    }
}


