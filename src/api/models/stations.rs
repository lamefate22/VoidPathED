use serde::{Deserialize, Serialize};

// API Path: api/stations (response)
#[derive(Debug, Serialize, Deserialize)]
pub struct FoundStation {
    pub name: String,
    pub system: String
}