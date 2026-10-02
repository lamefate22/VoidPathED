use crate::domain::event::ShipStatus;
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
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_route(&mut self, route: Vec<RouteStep>) {
        self.active_route = Some(route);
        self.current_step_index = 0;
        self.is_searching = false;
        self.last_error = None;
    }

    pub fn clear_route(&mut self) {
        self.active_route = None;
        self.current_step_index = 0;
    }

    pub fn next_step(&mut self) -> bool {
        if let Some(route) = &self.active_route
            && self.current_step_index + 1 < route.len()
        {
            self.current_step_index += 1;
            return true;
        }
        false
    }

    pub fn prev_step(&mut self) -> bool {
        if self.current_step_index > 0 {
            self.current_step_index -= 1;
            return true;
        }
        false
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
