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
                    tracing::info!("Location: {} @ {} (Docked)", system, st_name);
                } else {
                    tracing::info!(
                        "Location: {} ({})",
                        system,
                        if *docked { "Docked" } else { "In space" }
                    );
                }
                state.is_docked = *docked;
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::Jump { system } => {
                tracing::info!("Jumped to system: {}", system);
                state.current_system = system.clone();
                state.is_docked = false;
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::Docked { system, station } => {
                tracing::info!("Docked at: {} ({})", station, system);
                state.current_system = system.clone();
                state.current_station = station.clone();
                state.is_docked = true;

                let mut advanced = false;
                if auto_advance
                    && let Some(step) = state.current_step()
                    && step.destination.system.eq_ignore_ascii_case(system)
                    && step.destination.station.eq_ignore_ascii_case(station)
                {
                    // Check if player is carrying any unsold target commodities
                    let has_unsold_cargo = step.commodities.iter().any(|c| {
                        let is_sold = state.step_commodities_sold.contains(&c.name);
                        if is_sold {
                            return false;
                        }

                        let was_bought = state.step_commodities_bought.contains(&c.name);
                        let in_inventory = state.current_cargo.as_ref().map(|cargo| {
                            cargo.items.iter().any(|item| {
                                item.name.eq_ignore_ascii_case(&c.name)
                                    || item
                                        .name_localised
                                        .as_deref()
                                        .map(|l| l.eq_ignore_ascii_case(&c.name))
                                        .unwrap_or(false)
                            })
                        });

                        was_bought || in_inventory == Some(true)
                    });

                    if has_unsold_cargo {
                        tracing::info!(
                            "Docked at destination {} / {}, waiting for commodity sale",
                            system,
                            station
                        );
                    } else {
                        tracing::info!(
                            "Route auto-advance: arrived at target {} / {}",
                            system,
                            station
                        );
                        advanced = state.next_step();
                    }
                }
                EventOutcome {
                    changed: true,
                    step_advanced: advanced,
                    ship_updated: false,
                }
            }
            GameEvent::Undocked { .. } => {
                tracing::info!("Undocked from {}", state.current_station);
                state.is_docked = false;
                EventOutcome {
                    changed: true,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
            GameEvent::MarketSell {
                commodity, count, ..
            } => {
                let mut advanced = false;
                let mut changed = false;

                let matching_comm = state.current_step().and_then(|step| {
                    step.commodities
                        .iter()
                        .find(|c| c.name.eq_ignore_ascii_case(commodity))
                        .map(|c| c.name.clone())
                });

                if let Some(comm_name) = matching_comm {
                    state.step_commodities_sold.insert(comm_name);
                    tracing::info!(
                        "Sold target commodity '{}' ({}t) at {}",
                        commodity,
                        count,
                        state.current_system
                    );
                    changed = true;

                    // Switch to the next unsold commodity if available
                    let next_unsold_idx = state.current_step().and_then(|step| {
                        step.commodities
                            .iter()
                            .enumerate()
                            .find(|(_, c)| !state.step_commodities_sold.contains(&c.name))
                            .map(|(idx, _)| idx)
                    });
                    if let Some(idx) = next_unsold_idx {
                        state.current_commodity_index = idx;
                    }
                }

                if auto_advance {
                    let should_advance = state
                        .current_step()
                        .map(|step| {
                            let is_at_dest = step
                                .destination
                                .system
                                .eq_ignore_ascii_case(&state.current_system);
                            let all_sold = !step.commodities.is_empty()
                                && step
                                    .commodities
                                    .iter()
                                    .all(|c| state.step_commodities_sold.contains(&c.name));
                            is_at_dest && all_sold
                        })
                        .unwrap_or(false);

                    if should_advance {
                        tracing::info!(
                            "Route auto-advance: all target commodities sold for step {} -> advancing",
                            state.current_step_index + 1
                        );
                        advanced = state.next_step();
                    }
                }

                EventOutcome {
                    changed,
                    step_advanced: advanced,
                    ship_updated: false,
                }
            }
            GameEvent::Loadout(loadout) => {
                let name = loadout.ship_name.trim();
                let ship_desc = if name.is_empty() || name.eq_ignore_ascii_case(&loadout.ship_type)
                {
                    loadout.ship_type.clone()
                } else {
                    format!("\"{}\" ({})", name, loadout.ship_type)
                };
                tracing::info!(
                    "Ship: {} | Cargo: {}t | Jump: {:.1} LY | Pad: {:?}",
                    ship_desc,
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
                tracing::info!("Cargo: {}t on board", cargo.count);
                state.current_cargo = Some(cargo.clone());
                state.sync_cargo_with_current_step();
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
            GameEvent::MarketBuy { commodity, count } => {
                let mut changed = false;

                let matching_comm = state.current_step().and_then(|step| {
                    step.commodities
                        .iter()
                        .find(|c| c.name.eq_ignore_ascii_case(commodity))
                        .map(|c| c.name.clone())
                });

                if let Some(comm_name) = matching_comm {
                    state.step_commodities_bought.insert(comm_name);
                    tracing::info!("Target commodity bought: {} ({}t)", commodity, count);
                    changed = true;

                    // Switch to the next unbought commodity if available
                    let next_unbought_idx = state.current_step().and_then(|step| {
                        step.commodities
                            .iter()
                            .enumerate()
                            .find(|(_, c)| !state.step_commodities_bought.contains(&c.name))
                            .map(|(idx, _)| idx)
                    });
                    if let Some(idx) = next_unbought_idx {
                        state.current_commodity_index = idx;
                    }
                }

                EventOutcome {
                    changed,
                    step_advanced: false,
                    ship_updated: false,
                }
            }
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
