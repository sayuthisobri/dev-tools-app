use crate::states::AppState;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

type SharedAppState = Arc<Mutex<AppState>>;

/// Enum for common state fields to auto-map to events
#[derive(Debug, Clone, Copy)]
pub enum StateField {
    Dock,
    Window,
    Theme,
}

impl StateField {
    /// Maps field to its event name (extend as needed).
    pub fn event_name(self) -> &'static str {
        match self {
            StateField::Dock => "dock-updated",
            StateField::Window => "window-updated",
            StateField::Theme => "theme-updated",
        }
    }
}

/// Generic helper to update app state and emit only the specific field.
/// Reduces boilerplate and payload size by emitting targeted updates.
///
/// # Arguments
/// * `state` - The shared app state (locked internally).
/// * `app` - The Tauri app handle for emitting events.
/// * `field` - The state field to update and emit.
/// * `update_fn` - Closure that modifies the AppState.
///
/// # Returns
/// * `Ok(())` on success, or an error string on failure.
pub fn update_field_and_emit<F>(
    state: &State<'_, SharedAppState>,
    app: &AppHandle,
    field: StateField,
    update_fn: F,
) -> Result<(), String>
where
    F: FnOnce(&mut AppState),
{
    let mut state_guard = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    update_fn(&mut state_guard);

    // Emit only the specific field based on the enum
    match field {
        StateField::Dock => {
            let _ = app.emit(field.event_name(), &state_guard.dock);
        }
        StateField::Window => {
            let _ = app.emit(field.event_name(), &state_guard.window);
        }
        StateField::Theme => {
            let _ = app.emit(field.event_name(), &state_guard.theme);
        }
    }
    Ok(())
}