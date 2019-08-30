/// Basically this is a repimplementation of the get and insert methods on TypeMap.
/// 
/// The purpose of this is so I can have my own local key trait that I can implement on
/// any type I wish. This is needed because I need a stable type from std to use as a key,
/// due to different compilations changing the type id of a locally declared type.

use std::any::{Any, TypeId};

use unsafe_any::{UnsafeAny, UnsafeAnyExt};
use typemap;

use serenity::prelude::*;

use crate::toaster_framework::ToasterFramework;

pub trait ShareMapHackKey: Any
{
    type Value: Any;
}

pub trait HackKey: TypeMapKey
{
    type Value: Any;
}

impl ShareMapHackKey for ()
{
    type Value = ToasterFramework;
}

// impl TypeMapKey for dyn ShareMapHackKey<Value = ToasterFramework>
// {
//     type Value = ToasterFramework;
// }

impl<T: 'static> TypeMapKey for dyn ShareMapHackKey<Value=T>
{
    type Value = T;
}

pub trait KeyValueBounds = Any + UnsafeAny + Send + Sync;

pub trait ShareMapHack
{
    fn get_hack<K: ShareMapHackKey>(&self) -> Option<&K::Value>
    where
        K::Value: KeyValueBounds;

    fn insert_hack<K: ShareMapHackKey>(&mut self, item: K::Value)
    where
        K::Value: KeyValueBounds;
}

impl ShareMapHack for ShareMap
{
    fn get_hack<K: ShareMapHackKey>(&self) -> Option<&K::Value>
    where 
        K::Value: KeyValueBounds
    {
        let map = unsafe { self.data() };

        println!("Getting value from sharemap hack...");
        let out = map.get(&TypeId::of::<K>())
            .map(|b| unsafe { b.downcast_ref_unchecked::<K::Value>() });
        println!("Got value from sharemap hack, returning now...");

        out
    }

    fn insert_hack<K: ShareMapHackKey>(&mut self, item: K::Value)
    where
        K::Value: KeyValueBounds
    {
        let map = unsafe { self.data_mut() };

        let boxed: Box<dyn UnsafeAny + Send + Sync> = Box::new(item);
        map.insert(TypeId::of::<K>(), boxed)
            .map(|b| unsafe { *b.downcast_unchecked::<K::Value>() });
    }
}

pub trait ToasterHack<T>: ShareMapHack
where
    T: ShareMapHackKey<Value = ToasterFramework>
{
    fn get_toaster(&self) -> Option<ToasterFramework>
    {
        self.get_hack::<T>().cloned()
    }

    fn insert_toaster(&mut self, framework: ToasterFramework)
    {
        self.insert_hack::<T>(framework)
    }
}

impl ToasterHack<()> for ShareMap {}