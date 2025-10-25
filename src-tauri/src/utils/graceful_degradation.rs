use crate::errors::DockError;
use tracing::{error, warn};

pub trait GracefulDegradation<T> {
    fn with_fallback<F>(self, fallback: F) -> Result<T, crate::errors::DockError>
    where
        F: FnOnce() -> T,
        Self: Sized;
}

impl<T> GracefulDegradation<T> for Result<T, DockError> {
    fn with_fallback<F>(self, fallback: F) -> Result<T, DockError>
    where
        F: FnOnce() -> T,
    {
        match self {
            Ok(value) => {
                warn!("Operation succeeded but fallback was requested");
                Ok(value)
            }
            Err(e) => {
                error!("Operation failed, using fallback: {:?}", e);
                Ok(fallback())
            }
        }
    }
}

pub fn safe_dock_operation<F, T>(operation: F, fallback: T) -> T
where
    F: FnOnce() -> Result<T, DockError>,
{
    match operation() {
        Ok(result) => result,
        Err(e) => {
            error!("Dock operation failed, using fallback: {:?}", e);
            fallback
        }
    }
}