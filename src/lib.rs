#![feature(iter_collect_into)]

mod constants;

mod error;
mod utils;

mod models;

#[path = "commands/lib.rs"]
mod commands;

extern crate jaq_core;
extern crate serde;
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[path = "test_verman_schema.rs"]
mod tests;
