#![feature(try_trait_v2)]
#![feature(async_closure)]
#![feature(trait_alias)]
#![feature(iter_next_chunk)]
#![feature(decl_macro)]

#[cfg(not(target_family = "wasm"))]
pub mod server;

pub mod client;
pub mod shared;
