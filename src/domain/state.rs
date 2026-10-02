use std::collections::HashSet;

use crate::domain::event::{CargoHold, ShipLoadout, ShipStatus};
use crate::domain::route::RouteStep;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    pub active_route: Option<Vec<RouteStep>>,
    pub current_step_index: usize,
    pub current_system: String,
    pub current_station: String,
    pub is_docked: bool,
    pub journal_connected: bool,
    pub is_searching: bool,
    pub last_error: Option<String>,
    pub ship_status: ShipStatus,
    pub current_ship: Option<ShipLoadout>,
    pub current_cargo: Option<CargoHold>,
    pub click_through: bool,
    pub notification: Option<String>,

    // Multi-commodity tracking
    pub step_commodities_bought: HashSet<String>,
    pub step_commodities_sold: HashSet<String>,
    pub current_commodity_index: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_route(&mut self, route: Vec<RouteStep>) {
        self.active_route = Some(route);
        self.current_step_index = 0;
        self.current_commodity_index = 0;
        self.step_commodities_bought.clear();
        self.step_commodities_sold.clear();
        self.is_searching = false;
        self.last_error = None;
        self.sync_cargo_with_current_step();
    }

    pub fn clear_route(&mut self) {
        self.active_route = None;
        self.current_step_index = 0;
        self.current_commodity_index = 0;
        self.step_commodities_bought.clear();
        self.step_commodities_sold.clear();
    }

    pub fn next_step(&mut self) -> bool {
        if let Some(route) = &self.active_route
            && self.current_step_index + 1 < route.len()
        {
            self.current_step_index += 1;
            self.current_commodity_index = 0;
            self.step_commodities_bought.clear();
            self.step_commodities_sold.clear();
            self.sync_cargo_with_current_step();
            return true;
        }
        false
    }

    pub fn prev_step(&mut self) -> bool {
        if self.current_step_index > 0 {
            self.current_step_index -= 1;
            self.current_commodity_index = 0;
            self.step_commodities_bought.clear();
            self.step_commodities_sold.clear();
            self.sync_cargo_with_current_step();
            return true;
        }
        false
    }

    pub fn cycle_commodity(&mut self) {
        if let Some(step) = self.current_step()
            && !step.commodities.is_empty()
        {
            self.current_commodity_index =
                (self.current_commodity_index + 1) % step.commodities.len();
        }
    }

    pub fn sync_cargo_with_current_step(&mut self) {
        let matching_names: Vec<String> =
            if let (Some(step), Some(cargo)) = (self.current_step(), &self.current_cargo) {
                step.commodities
                    .iter()
                    .filter(|comm| {
                        cargo.items.iter().any(|item| {
                            item.name.eq_ignore_ascii_case(&comm.name)
                                || item
                                    .name_localised
                                    .as_deref()
                                    .map(|l| l.eq_ignore_ascii_case(&comm.name))
                                    .unwrap_or(false)
                        })
                    })
                    .map(|c| c.name.clone())
                    .collect()
            } else {
                Vec::new()
            };

        for name in matching_names {
            self.step_commodities_bought.insert(name);
        }
    }

    pub fn current_step(&self) -> Option<&RouteStep> {
        self.active_route
            .as_ref()
            .and_then(|r| r.get(self.current_step_index))
    }

    pub fn total_steps(&self) -> usize {
        self.active_route.as_ref().map(|r| r.len()).unwrap_or(0)
    }

    pub fn total_route_profit(&self) -> u64 {
        self.active_route
            .as_ref()
            .map(|r| r.iter().map(|s| s.total_profit).sum())
            .unwrap_or(0)
    }
}
