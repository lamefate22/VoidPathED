use serde::{Deserialize, Serialize};

// API Path: api/results
#[derive(Debug, Serialize, Deserialize)]
pub struct TradeRouteResult {
    pub job: String,
    pub state: String,
    pub status: String,
    pub result: Option<Vec<RouteStep>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteStep {
    pub commodities: Vec<Commodity>,
    pub destination: SystemInfo,
    pub cumulative_profit: u64,
    pub source: SystemInfo,
    pub total_profit: u64,
    pub distance: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commodity {
    pub destination_commodity: PriceInfo,
    pub source_commodity: PriceInfo,
    pub total_profit: u64,
    pub name: String,
    pub amount: u16,
    pub profit: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceInfo {
    pub sell_price: u32,
    pub buy_price: u32,
    pub demand: u32,
    pub supply: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub system: String,
    pub station: String,
    pub distance_to_arrival: u32
}
