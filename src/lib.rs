#![feature(try_trait_v2)]

use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
pub mod server;

#[derive(Serialize, Deserialize)]
pub struct Flag {
    pub name: String,
    pub enabled: bool,
}
