use aws_sdk_s3::error::SdkError;
use serde::Serialize;

pub type AwsResult<T> = anyhow::Result<T, AwsError>;
#[derive(Debug, thiserror::Error, Serialize)]
pub enum AwsError {
    #[error("[Config] {0}")]
    Config(String),

    #[error("[General] {0}")]
    General(String),

    #[error("[IO] {0}")]
    Io(String),

    #[error("[Timeout] {0}")]
    Timeout(String),

    #[error("[Serialization] {0}")]
    Serialization(String),

    #[error("[BucketNotFound] {0}")]
    S3BucketNotFound(String),

    #[error("[Invalid path] {0}")]
    InvalidPath(String),

    #[error("[Profile] {0}, {1}")]
    AwsProfile(String, String),
}

impl<E, R> From<SdkError<E, R>> for AwsError {
    fn from(value: SdkError<E, R>) -> Self {
        match &value {
            SdkError::DispatchFailure(f) => {
                if f.is_io(){
                    AwsError::Io(value.to_string())
                }else if f.is_timeout(){
                    AwsError::Timeout(value.to_string())
                }else{
                    AwsError::General(value.to_string())
                }
            }
            _ => AwsError::General(value.to_string()),
        }
    }
}
