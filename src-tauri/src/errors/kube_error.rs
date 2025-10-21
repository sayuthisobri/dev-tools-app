use kube::config::KubeconfigError;
use kube::Error;
use serde::Serialize;

pub type KubeResult<T> = anyhow::Result<T, KubeError>;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum KubeError {
    #[error("[Kube] {0}")]
    Kube(String),
    #[error("[Config] {0}")]
    Kubeconfig(String),
    #[error("[Auth] {0}")]
    KubeAuth(String),
}

impl From<kube::Error> for KubeError {
    fn from(error: kube::Error) -> Self {
        match error {
            Error::Auth(_) => KubeError::KubeAuth(error.to_string().replace("auth error: ", "")),
            _ => KubeError::Kube(error.to_string()),
        }
    }
}

impl From<KubeconfigError> for KubeError {
    fn from(error: KubeconfigError) -> Self {
        KubeError::Kubeconfig(error.to_string())
    }
}
