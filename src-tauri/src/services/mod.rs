#![allow(dead_code)]

pub mod aws;
pub mod aws_s3;
pub mod dock_progress;
pub mod http_request;
pub mod kube;
pub mod kube_config;
pub mod kube_log;
pub mod request;
pub mod shell;


pub mod commands {
    pub use crate::services::aws::commands::*;
    pub use crate::services::aws_s3::commands::*;
    pub use crate::services::dock_progress::commands::*;
    pub use crate::services::http_request::commands::*;
    pub use crate::services::kube_config::commands::*;
    use tauri::generate_handler;
    use tauri::ipc::Invoke;

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
        ]
    }
}
