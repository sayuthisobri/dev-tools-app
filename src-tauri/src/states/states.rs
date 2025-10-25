use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub window: WindowState,
    pub theme: String,
    pub dock: DockState,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub monitor_name: String,
    pub scale_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DockState {
    pub progress: Option<f64>,
    pub badge: Option<String>,
    pub progress_color: Option<String>,
}

impl AppState {
    /// Helper to emit for a specific field (ties field to event and emits only that field).
    /// Usage: state.emit_for_field(app, StateField::Dock, |s| s.dock.progress = Some(0.5))
    #[allow(dead_code)]
    pub fn emit_for_field<F>(
        &mut self,
        app: &tauri::AppHandle,
        field: crate::utils::state_emitter::StateField,
        update_fn: F,
    ) where
        F: FnOnce(&mut Self),
    {
        update_fn(self);
        match field {
            crate::utils::state_emitter::StateField::Dock => {
                let _ = app.emit(field.event_name(), &self.dock);
            }
            crate::utils::state_emitter::StateField::Window => {
                let _ = app.emit(field.event_name(), &self.window);
            }
            crate::utils::state_emitter::StateField::Theme => {
                let _ = app.emit(field.event_name(), &self.theme);
            }
        }
    }
}
