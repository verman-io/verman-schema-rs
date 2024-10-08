#![feature(iter_collect_into)]
#![feature(pattern)]
#![feature(try_trait_v2)]

extern crate core;

pub mod models;

pub mod verman_schema;

#[path = "task/lib.rs"]
pub mod task;

#[path = "commands/lib.rs"]
pub mod commands;
mod errors;

#[path = "pipeline/lib.rs"]
pub mod pipeline;

#[cfg(test)]
mod test_models;
