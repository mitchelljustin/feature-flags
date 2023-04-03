#![feature(try_trait_v2)]
#![feature(async_closure)]
#![feature(trait_alias)]

#[cfg(not(target_family = "wasm"))]
pub mod server;

pub mod client;
pub mod shared;
