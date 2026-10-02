use crate::domain::event::{GameEvent, ShipStatus};
use crate::domain::state::AppState;

pub struct GameTracker;

impl GameTracker {
    /// Applies a GameEvent to the AppState, performing auto-advance checks when enabled.
    /// Returns true if the state changed or advanced.
    pub fn handle_event(state: &mut AppState, event: &GameEvent, auto_advance: bool) -> bool {
        state.journal_connected = true;

        match event {
            GameEvent::Location {
                system,
                station,
                docked,
            } => {
                state.current_system = system.clone();
                if let Some(st_name) = station {
                    state.current_station = st_name.clone();
                }
                state.is_docked = *docked;
                true
            }
            GameEvent::Jump { system } => {
                state.current_system = system.clone();
                state.is_docked = false;
                true
            }
            GameEvent::Docked { system, station } => {
                state.current_system = system.clone();
                state.current_station = station.clone();
                state.is_docked = true;

                if auto_advance
                    && let Some(step) = state.current_step()
                    && step.destination.system.eq_ignore_ascii_case(system)
                    && step.destination.station.eq_ignore_ascii_case(station)
                {
                    tracing::info!(
                        "Auto-advancing route hop on arrival at target: {} / {}",
                        system,
                        station
                    );
                    state.next_step();
                }
                true
            }
            GameEvent::Undocked { .. } => {
                state.is_docked = false;
                true
            }
            GameEvent::MarketSell { commodity, .. } => {
                if auto_advance
                    && let Some(step) = state.current_step()
                    && step
                        .destination
                        .system
                        .eq_ignore_ascii_case(&state.current_system)
                {
                    let is_matching_commodity = step
                        .commodities
                        .iter()
                        .any(|c| c.name.eq_ignore_ascii_case(commodity));

                    if is_matching_commodity {
                        tracing::info!(
                            "Auto-advancing route hop on selling target commodity '{}' at {}",
                            commodity,
                            state.current_system
                        );
                        state.next_step();
                        return true;
                    }
                }
                false
            }
            GameEvent::Status(status) => Self::handle_status(state, *status),
            GameEvent::MarketBuy { .. } => false,
        }
    }

    /// Updates ship status flags from Status.json
    pub fn handle_status(state: &mut AppState, status: ShipStatus) -> bool {
        if state.ship_status == status {
            return false;
        }

        state.ship_status = status;
        if status.docked {
            state.is_docked = true;
        } else if status.supercruise || status.in_hyperspace {
            state.is_docked = false;
        }
        true
    }
}
