extern crate oxygengine_core as core;

pub mod ast;
pub mod vm;

use core::id::ID;

pub type GUID = ID<()>;
