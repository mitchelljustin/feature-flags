use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flag {
    pub name: String,
    pub enabled: bool,
}
