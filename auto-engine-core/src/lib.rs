use wasmtime::component::bindgen;

bindgen!({world: "plugin", path: "wit/node.wit"});

pub use plugins::*;

#[cfg(feature = "context")]
pub mod context;
#[cfg(feature = "event")]
pub mod event;
#[cfg(feature = "types")]
pub mod node;
#[cfg(feature = "pipeline")]
pub mod pipeline;
#[cfg(feature = "runner")]
pub mod runner;
#[cfg(feature = "types")]
pub mod schema;
#[cfg(feature = "types")]
pub mod types;
#[cfg(feature = "utils")]
pub mod utils;

pub mod workflow;

#[cfg(feature = "types")]
pub mod register;

#[cfg(feature = "wasm")]
pub mod plugin;

mod action;
pub mod notification;
pub mod mcp;
