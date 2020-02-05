extern crate oxygengine_core as core;

pub mod ast;
pub mod vm;

#[cfg(test)]
mod tests;

use core::id::ID;

pub type GUID = ID<()>;
