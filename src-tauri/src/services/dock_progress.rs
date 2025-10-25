pub mod commands {
    use crate::errors::DockError;
    use crate::states::AppState;
    use crate::utils::graceful_degradation::safe_dock_operation;
    use crate::utils::progress_helper::{
        self, clear_dock_progress_async, set_dock_progress_fraction_async,
    };
    use crate::utils::state_emitter::{update_field_and_emit, StateField};
    use std::sync::{Arc, Mutex};
    use tauri::State;
    use tauri::command;

    type SharedAppState = Arc<Mutex<AppState>>;

    #[command]
    pub fn set_dock_progress(
        progress: f64,
        state: State<SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        // Input validation
        if !(0.0..=1.0).contains(&progress) {
            let error = DockError::invalid_progress(progress, "Progress must be between 0.0 and 1.0");
            log::error!("{:?}", error);
            return Err(error.to_string());
        }

        let state_guard = state.lock().unwrap();
        let color = state_guard.dock.progress_color.clone();

        let result = safe_dock_operation(
            || {
                progress_helper::set_dock_progress_fraction(progress, color)
                    .map_err(|e| DockError::general(e.to_string(), "set_dock_progress_fraction"))
            },
            (), // No-op fallback
        );

        match update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.progress = Some(progress)) {
            Ok(_) => Ok(result),
            Err(e) => {
                log::error!("Failed to update state: {}", e);
                Err(e)
            }
        }
    }

    #[command]
    pub fn clear_dock(state: State<SharedAppState>, app: tauri::AppHandle) -> Result<(), String> {
        log::info!("Clearing dock progress");
        let result = progress_helper::clear_dock_progress().map_err(|e| e.to_string())?;
        update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.progress = None)?;
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

        // Extract color once outside the loop
        let color = state.lock().unwrap().dock.progress_color.clone();

        // Show progress from 0% to 100% in steps
        for i in 0..=20 {
            let progress = i as f64 / 20.0;
            log::debug!("Setting progress to {:.2}", progress);

            match set_dock_progress_fraction_async(progress, color.clone()).await {
                Ok(_) => log::debug!("Successfully set progress to {:.2}", progress),
                Err(e) => {
                    log::error!("Failed to set dock progress to {:.2}: {:?}", progress, e);
                    return Err(format!("Failed to set progress: {}", e));
                }
            }

            // Update state and emit event
            update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.progress = Some(progress))?;

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

        // Update state to clear progress
        update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.progress = None)?;

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
        update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.badge = Some(label.to_string()))?;
        Ok(result)
    }

    #[command]
    pub fn clear_dock_badge(
        state: State<SharedAppState>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        let result = progress_helper::clear_dock_badge().map_err(|e| e.to_string())?;
        update_field_and_emit(&state, &app, StateField::Dock, |s| s.dock.badge = None)?;
        Ok(result)
    }
    #[command]
    pub fn simulate_panic() -> Result<(), String> {
        log::info!("Simulating panic for crash testing");

        // Log additional context before panic
        let thread_info = format!("Thread: {:?}", std::thread::current());
        log::error!("Intentional panic triggered. Context: {}", thread_info);

        panic!("Simulated panic to test crash logging behavior");
    }
}
