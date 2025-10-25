use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DockError {
    #[error("Dock operation failed: {message} (context: {context})")]
    General { message: String, context: String },
    #[error("Icon loading error: {message} (path: {path:?})")]
    IconLoad { message: String, path: Option<String> },
    #[error("Objective-C binding error: {message} (selector: {selector:?})")]
    ObjectiveC { message: String, selector: Option<String> },
    #[error("Invalid progress value: {value} (reason: {reason})")]
    InvalidProgress { value: f64, reason: String },
    #[error("Async operation failed: {message} (operation: {operation})")]
    AsyncOperation { message: String, operation: String },
    #[error("Queue processing error: {message} (queue_size: {queue_size})")]
    QueueError { message: String, queue_size: usize },
    #[error("State lock error: {0}")]
    StateLock(String),
}

impl DockError {
    pub fn general(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            context: context.into()
        }
    }

    pub fn icon_load(message: impl Into<String>, path: Option<String>) -> Self {
        Self::IconLoad {
            message: message.into(),
            path
        }
    }

    pub fn objective_c(message: impl Into<String>, selector: Option<String>) -> Self {
        Self::ObjectiveC {
            message: message.into(),
            selector
        }
    }

    pub fn invalid_progress(value: f64, reason: impl Into<String>) -> Self {
        Self::InvalidProgress {
            value,
            reason: reason.into()
        }
    }

    pub fn async_operation(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::AsyncOperation {
            message: message.into(),
            operation: operation.into()
        }
    }

    pub fn queue_error(message: impl Into<String>, queue_size: usize) -> Self {
        Self::QueueError {
            message: message.into(),
            queue_size
        }
    }

    pub fn state_lock(message: impl Into<String>) -> Self {
        Self::StateLock(message.into())
    }
}

pub type DockResult<T> = Result<T, DockError>;

impl From<String> for DockError {
    fn from(error: String) -> Self {
        DockError::general(error, "string conversion")
    }
}

impl From<std::io::Error> for DockError {
    fn from(error: std::io::Error) -> Self {
        DockError::general(error.to_string(), "io error")
    }
}