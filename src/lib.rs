#![feature(try_trait_v2)]
#![feature(iter_collect_into)]
extern crate core;

pub mod models;

#[path = "task/lib.rs"]
pub mod task;

#[path = "commands/lib.rs"]
pub mod commands;
mod errors;

#[path = "pipeline/lib.rs"]
pub mod pipeline;

#[cfg(test)]
mod test_models;
