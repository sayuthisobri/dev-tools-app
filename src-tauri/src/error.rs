use anyhow::Error;
use serde::Serialize;
use tauri::http::uri::InvalidUri;
use zip::result::ZipError;

#[derive(Debug, Clone, Serialize)]
pub struct APIError {
    message: String,
    category: String,
}
impl From<reqwest::Error> for APIError {
    fn from(error: reqwest::Error) -> Self {
        APIError {
            message: error.to_string(),
            category: "http".to_string(),
        }
    }
}
impl From<InvalidUri> for APIError {
    fn from(error: InvalidUri) -> Self {
        APIError {
            message: error.to_string(),
            category: "invalidUri".to_string(),
        }
    }
}
impl From<reqwest::header::InvalidHeaderValue> for APIError {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        APIError {
            message: error.to_string(),
            category: "invalidHeader".to_string(),
        }
    }
}

impl From<reqwest::header::InvalidHeaderName> for APIError {
    fn from(error: reqwest::header::InvalidHeaderName) -> Self {
        APIError {
            message: error.to_string(),
            category: "invalidHeaderName".to_string(),
        }
    }
}

impl From<reqwest::header::ToStrError> for APIError {
    fn from(error: reqwest::header::ToStrError) -> Self {
        APIError {
            message: error.to_string(),
            category: "toStrError".to_string(),
        }
    }
}

impl From<std::io::Error> for APIError {
    fn from(error: std::io::Error) -> Self {
        APIError {
            message: error.to_string(),
            category: "io".to_string(),
        }
    }
}
impl From<cookie_store::Error> for APIError {
    fn from(error: cookie_store::Error) -> Self {
        APIError {
            message: error.to_string(),
            category: "cookieStore".to_string(),
        }
    }
}

impl From<url::ParseError> for APIError {
    fn from(error: url::ParseError) -> Self {
        APIError {
            message: error.to_string(),
            category: "urlParse".to_string(),
        }
    }
}

impl From<cookie_store::CookieError> for APIError {
    fn from(error: cookie_store::CookieError) -> Self {
        APIError {
            message: error.to_string(),
            category: "cookieStore".to_string(),
        }
    }
}

impl From<serde_json::Error> for APIError {
    fn from(error: serde_json::Error) -> Self {
        APIError {
            message: error.to_string(),
            category: "serdeJson".to_string(),
        }
    }
}

impl From<base64::DecodeError> for APIError {
    fn from(error: base64::DecodeError) -> Self {
        APIError {
            message: error.to_string(),
            category: "base64".to_string(),
        }
    }
}

impl From<ZipError> for APIError {
    fn from(error: ZipError) -> Self {
        APIError {
            message: error.to_string(),
            category: "zip".to_string(),
        }
    }
}
impl From<cookie::ParseError> for APIError {
    fn from(error: cookie::ParseError) -> Self {
        APIError {
            message: error.to_string(),
            category: "cookie".to_string(),
        }
    }
}
impl From<Error> for APIError {
    fn from(error: Error) -> Self {
        APIError {
            message: error.to_string(),
            category: error.source().map_or("general".into(), |s| s.to_string()),
        }
    }
}
