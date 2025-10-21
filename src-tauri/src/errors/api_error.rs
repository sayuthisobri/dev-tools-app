use crate::errors::kube_error::KubeError;
use crate::errors::AwsError;
use anyhow::Error;
use serde::Serialize;
use tauri::http::uri::InvalidUri;
use zip::result::ZipError;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum APIError {
    #[error("General: {0}")]
    General(String),
    #[error("HTTP: {0}")]
    Http(String),
    #[error("IO: {0}")]
    Cookie(String),
    #[error("Cookie: {0}")]
    Io(String),
    #[error("Parser: {0}")]
    Parser(String),
    #[error("Zip: {0}")]
    Zip(String),
    #[error("AWS: {0}")]
    Aws(AwsError),
    #[error("Kube: {0}")]
    Kube(KubeError),
}

pub type ApiResult<T> = anyhow::Result<T, APIError>;

impl From<reqwest::Error> for APIError {
    fn from(error: reqwest::Error) -> Self {
        APIError::Http(error.to_string())
    }
}
impl From<InvalidUri> for APIError {
    fn from(error: InvalidUri) -> Self {
        APIError::Http(error.to_string())
    }
}
impl From<reqwest::header::InvalidHeaderValue> for APIError {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        APIError::Http(error.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderName> for APIError {
    fn from(error: reqwest::header::InvalidHeaderName) -> Self {
        APIError::Http(error.to_string())
    }
}

impl From<reqwest::header::ToStrError> for APIError {
    fn from(error: reqwest::header::ToStrError) -> Self {
        APIError::Http(error.to_string())
    }
}

impl From<std::io::Error> for APIError {
    fn from(error: std::io::Error) -> Self {
        APIError::Io(error.to_string())
    }
}
impl From<cookie_store::Error> for APIError {
    fn from(error: cookie_store::Error) -> Self {
        APIError::Cookie(error.to_string())
    }
}

impl From<url::ParseError> for APIError {
    fn from(error: url::ParseError) -> Self {
        APIError::Parser(error.to_string())
    }
}

impl From<cookie_store::CookieError> for APIError {
    fn from(error: cookie_store::CookieError) -> Self {
        APIError::Cookie(error.to_string())
    }
}

impl From<serde_json::Error> for APIError {
    fn from(error: serde_json::Error) -> Self {
        APIError::Parser(error.to_string())
    }
}

impl From<base64::DecodeError> for APIError {
    fn from(error: base64::DecodeError) -> Self {
        APIError::Parser(error.to_string())
    }
}

impl From<ZipError> for APIError {
    fn from(error: ZipError) -> Self {
        APIError::Zip(error.to_string())
    }
}
impl From<cookie::ParseError> for APIError {
    fn from(error: cookie::ParseError) -> Self {
        APIError::Cookie(error.to_string())
    }
}
impl From<Error> for APIError {
    fn from(error: Error) -> Self {
        APIError::Io(error.to_string())
    }
}

impl From<String> for APIError {
    fn from(error: String) -> Self {
        APIError::General(error)
    }
}

impl From<AwsError> for APIError {
    fn from(error: AwsError) -> Self {
        APIError::Aws(error)
    }
}
