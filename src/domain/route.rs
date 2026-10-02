use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoundStation {
    pub name: String,
    pub system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteStep {
    pub commodities: Vec<Commodity>,
    pub destination: SystemInfo,
    pub cumulative_profit: u64,
    pub source: SystemInfo,
    pub total_profit: u64,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commodity {
    pub destination_commodity: PriceInfo,
    pub source_commodity: PriceInfo,
    pub total_profit: u64,
    pub name: String,
    pub amount: u16,
    pub profit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PriceInfo {
    pub sell_price: u32,
    pub buy_price: u32,
    pub demand: u32,
    pub supply: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub system: String,
    pub station: String,
    pub distance_to_arrival: Option<u32>,
}
