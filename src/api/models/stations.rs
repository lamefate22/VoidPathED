use serde::{Deserialize, Serialize};

// API Path: api/stations (response)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoundStation {
    pub name: String,
    pub system: String,
}