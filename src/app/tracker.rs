use crate::domain::event::{GameEvent, ShipStatus};
use crate::domain::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventOutcome {
    pub changed: bool,
    pub step_advanced: bool,
    pub ship_updated: bool,
}

pub struct GameTracker;

impl GameTracker {
    /// Applies a GameEvent to the AppState, performing auto-advance checks when enabled.
    /// Returns EventOutcome indicating what changed and whether hop advanced.
    pub fn handle_event(
        state: &mut AppState,
        event: &GameEvent,
        auto_advance: bool,
    ) -> EventOutcome {
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
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::Jump { system } => {
                state.current_system = system.clone();
                state.is_docked = false;
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::Docked { system, station } => {
                state.current_system = system.clone();
                state.current_station = station.clone();
                state.is_docked = true;

                let mut advanced = false;
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
                    advanced = state.next_step();
                }
                EventOutcome {
                    changed: true,
                    step_advanced: advanced,
                    ship_updated: false,
                }
            }
            GameEvent::Undocked { .. } => {
                state.is_docked = false;
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
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
                        let advanced = state.next_step();
                        return EventOutcome {
                            changed: true,
                            step_advanced: advanced,
                            ship_updated: false,
                        };
                    }
                }
                EventOutcome::default()
            }
            GameEvent::Loadout(loadout) => {
                tracing::info!(
                    "Player ship loadout detected: {} ({}) - Cargo: {}t, Jump: {:.1} LY, Pad: {:?}",
                    loadout.ship_name,
                    loadout.ship_type,
                    loadout.cargo_capacity,
                    loadout.max_jump_range,
                    loadout.pad_size
                );
                state.current_ship = Some(loadout.clone());
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: true,
                }
            }
            GameEvent::Cargo(cargo) => {
                tracing::info!("Player cargo hold updated: {} items on board", cargo.count);
                state.current_cargo = Some(cargo.clone());
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::Status(status) => {
                let changed = Self::handle_status(state, *status);
                EventOutcome {
                    changed,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::MarketBuy { .. } => EventOutcome::default(),
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
