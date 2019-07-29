// Used for group construction macro
#![feature(proc_macro_hygiene)]

#[macro_use]
mod utils;

mod groups;
pub use groups::framework_factory;

