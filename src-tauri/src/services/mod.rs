#![allow(dead_code)]

pub mod aws;
pub mod aws_s3;
pub mod dock_progress;
pub mod http;
pub mod kube;
pub mod kube_config;
pub mod kube_log;
pub mod request;
pub mod shell;

pub mod commands {
    use crate::errors::{APIError, ApiResult};
    pub use crate::services::aws::commands::*;
    pub use crate::services::aws_s3::commands::*;
    pub use crate::services::dock_progress::commands::*;
    pub use crate::services::http::commands::*;
    pub use crate::services::kube_config::commands::*;
    use std::env;
    use tauri::ipc::Invoke;
    use tauri::{command, generate_handler};

    #[command(async)]
    pub async fn env() -> ApiResult<Vec<String>> {
        log::info!("invoke env");
        let path_var = env::var("PATH").map_err(|e| {
            log::error!("Failed to get PATH environment variable: {}", e);
            APIError::General("Environment variable PATH not found".to_string())
        })?;
        let env_list: Vec<String> = path_var.split(':').map(String::from).collect();
        log::info!("env: {:?}", env_list);
        Ok(env_list)
    }

    pub fn setup_handler() -> fn(Invoke) -> bool {
        generate_handler![
            http_send_request,
            load_kube_config,
            aws_profiles,
            aws_s3_buckets,
            aws_s3_objects,
            set_dock_progress,
            clear_dock,
            test_dock_progress,
            set_dock_badge,
            simulate_panic,
            clear_dock_badge,
            env,
        ]
    }
}
