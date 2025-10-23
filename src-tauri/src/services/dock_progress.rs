pub mod commands {
    use crate::utils::progress_helper;
    use tauri::command;

    #[command]
    pub fn set_dock_progress(progress: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&progress) {
            return Err("Progress must be between 0.0 and 1.0".to_string());
        }
        progress_helper::set_dock_progress_fraction(progress);
        Ok(())
    }

    #[command]
    pub fn clear_dock() -> Result<(), String> {
        progress_helper::clear_dock_progress();
        Ok(())
    }

    // Test the dock progress with a simple animation
    #[command]
    pub async fn test_dock_progress() -> Result<(), String> {
        use std::time::Duration;
        use tokio::time::sleep;

        // Show progress from 0% to 100% in steps
        for i in 0..=20 {
            let progress = i as f64 / 20.0;
            set_dock_progress(progress)?;
            sleep(Duration::from_millis(100)).await;
        }

        // Keep it at 100% for a moment
        sleep(Duration::from_secs(1)).await;

        // Clear the progress
        clear_dock()?;

        Ok(())
    }

    #[command]
    pub fn set_dock_badge(label: &str) -> Result<(), String> {
        progress_helper::set_dock_badge(label);
        Ok(())
    }

    #[command]
    pub fn clear_dock_badge() -> Result<(), String> {
        progress_helper::clear_dock_badge();
        Ok(())
    }
}
