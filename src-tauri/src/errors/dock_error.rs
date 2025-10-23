use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DockError {
    #[error("Dock operation failed: {0}")]
    General(String),
    #[error("Icon loading error: {0}")]
    IconLoad(String),
    #[error("Objective-C binding error: {0}")]
    ObjectiveC(String),
    #[error("Invalid progress value: {0}")]
    InvalidProgress(String),
}

pub type DockResult<T> = Result<T, DockError>;

impl From<String> for DockError {
    fn from(error: String) -> Self {
        DockError::General(error)
    }
}