use crate::contract::ClipboardService;
use crate::domain::state::AppState;
use crate::error::CoreError;

pub struct ActionHandler;

impl ActionHandler {
    pub fn next_hop(state: &mut AppState) -> bool {
        state.next_step()
    }

    pub fn prev_hop(state: &mut AppState) -> bool {
        state.prev_step()
    }

    pub fn clear_route(state: &mut AppState) {
        state.clear_route();
    }

    pub fn copy_target_system(
        state: &AppState,
        clipboard: &dyn ClipboardService,
    ) -> Result<String, CoreError> {
        if let Some(step) = state.current_step() {
            let target_system = step.destination.system.clone();
            clipboard.set_text(&target_system)?;
            Ok(target_system)
        } else {
            Err(CoreError::Clipboard(
                "No active route step to copy".to_string(),
            ))
        }
    }

    pub fn get_detected_location(state: &AppState) -> Option<(String, String)> {
        if state.current_system.is_empty() {
            None
        } else {
            Some((state.current_system.clone(), state.current_station.clone()))
        }
    }
}
