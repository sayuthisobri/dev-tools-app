pub mod commands {
    use crate::states::AppState;
    use crate::utils::progress_helper::{
        self, clear_dock_progress_async, set_dock_progress_fraction_async,
    };
    use std::sync::{Arc, Mutex};
    use tauri::State;
    use tauri::{command, Emitter};

    type SharedAppState = Arc<Mutex<AppState>>;

    #[command]
    pub fn set_dock_progress(
        progress: f64,
        state: State<SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        log::info!("Setting dock progress to {}", progress);
        let result =
            progress_helper::set_dock_progress_fraction(progress).map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.progress = Some(progress);
        let _ = app.emit("dock-progress-updated", &state.dock);
        Ok(result)
    }

    #[command]
    pub fn clear_dock(state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        log::info!("Clearing dock progress");
        let result = progress_helper::clear_dock_progress().map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.progress = None;
        let _ = app.emit("dock-progress-updated", &state.dock);
        Ok(result)
    }

    // Test the dock progress with a simple animation
    #[command]
    pub async fn test_dock_progress(
        state: State<'_, SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        use std::time::Duration;
        use tokio::time::sleep;
        log::info!("Starting test_dock_progress animation");

        // Show progress from 0% to 100% in steps
        for i in 0..=20 {
            let progress = i as f64 / 20.0;
            log::debug!("Setting progress to {:.2}", progress);

            match set_dock_progress_fraction_async(progress).await {
                Ok(_) => log::debug!("Successfully set progress to {:.2}", progress),
                Err(e) => {
                    log::error!("Failed to set dock progress to {:.2}: {:?}", progress, e);
                    return Err(format!("Failed to set progress: {}", e));
                }
            }

            // Update state and emit event (acquire lock only for this block)
            {
                let mut state_guard = state.lock().unwrap();
                state_guard.dock.progress = Some(progress);
                let _ = app.emit("dock-progress-updated", &state_guard.dock);
            }

            sleep(Duration::from_millis(100)).await;
        }

        // Keep it at 100% for a moment
        log::info!("Progress animation complete, holding at 100%");
        sleep(Duration::from_secs(1)).await;

        // Clear the progress
        log::debug!("Clearing dock progress");
        match clear_dock_progress_async().await {
            Ok(_) => log::debug!("Successfully cleared dock progress"),
            Err(e) => {
                log::error!("Failed to clear dock progress: {:?}", e);
                return Err(format!("Failed to clear progress: {}", e));
            }
        }

        // Update state to clear progress (acquire lock only for this block)
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.dock.progress = None;
            let _ = app.emit("dock-progress-updated", &state_guard.dock);
        }

        log::info!("test_dock_progress completed successfully");
        Ok(())
    }

    #[command]
    pub fn set_dock_badge(
        label: &str,
        state: State<SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let result = progress_helper::set_dock_badge(label).map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.badge = Some(label.to_string());
        let _ = app.emit("dock-badge-updated", &state.dock);
        Ok(result)
    }

    #[command]
    pub fn clear_dock_badge(
        state: State<SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let result = progress_helper::clear_dock_badge().map_err(|e| e.to_string())?;
        let mut state = state.lock().unwrap();
        state.dock.badge = None;
        let _ = app.emit("dock-badge-updated", &state.dock);
        Ok(result)
    }
}
