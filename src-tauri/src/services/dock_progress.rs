pub mod commands {
    use crate::states::AppState;
    use crate::utils::progress_helper;
    use std::sync::{Arc, Mutex};
    use tauri::State;
    use tauri::{command, Emitter};

    type SharedAppState = Arc<Mutex<AppState>>;

    #[command]
    pub fn set_dock_progress(progress: f64, state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        let result = progress_helper::set_dock_progress_fraction(progress).map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.progress = Some(progress);
        let _ = app.emit("dock-progress-updated", &state.dock);
        Ok(result)
    }

    #[command]
    pub fn clear_dock(state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        let result = progress_helper::clear_dock_progress().map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.progress = None;
        let _ = app.emit("dock-progress-updated", &state.dock);
        Ok(result)
    }

    // Test the dock progress with a simple animation
    #[command]
    pub async fn test_dock_progress(state: State<'_, SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        use std::time::Duration;
        use tokio::time::sleep;

        // Show progress from 0% to 100% in steps
        for i in 0..=20 {
            let progress = i as f64 / 20.0;
            set_dock_progress(progress, state.clone(), app.clone())?;
            sleep(Duration::from_millis(100)).await;
        }

        // Keep it at 100% for a moment
        sleep(Duration::from_secs(1)).await;

        // Clear the progress
        clear_dock(state.clone(), app.clone())?;

        Ok(())
    }

    #[command]
    pub fn set_dock_badge(label: &str, state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        let result = progress_helper::set_dock_badge(label).map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.badge = Some(label.to_string());
        let _ = app.emit("dock-badge-updated", &state.dock);
        Ok(result)
    }

    #[command]
    pub fn clear_dock_badge(state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        let result = progress_helper::clear_dock_badge().map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.badge = None;
        let _ = app.emit("dock-badge-updated", &state.dock);
        Ok(result)
    }
}
