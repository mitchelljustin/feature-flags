#![feature(try_trait_v2)]
#![feature(fn_traits)]

#[cfg(not(target_family = "wasm"))]
pub mod server;

pub mod client;

pub mod shared;
