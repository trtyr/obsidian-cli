//! Kernel — core modules that provide fundamental vault operations.
//!
//! These modules are the foundation. Plugins build on top of them.

pub mod config;
pub mod fs;
pub mod index;
pub mod note;
pub mod output;
pub mod search;
pub mod vault;
