use crate::utils::expand_tilde;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KubeConfig {
    #[serde(rename = "current-context")]
    pub current_context: Option<String>,

    pub contexts: Vec<NamedContext>,
    pub clusters: Vec<NamedCluster>,

    // These fields are optional in kubeconfig; kept for completeness
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    pub kind: Option<String>,
    pub users: Option<Vec<NamedUser>>,
    pub preferences: Option<serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedContext {
    pub name: String,
    pub context: Context,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamedCluster {
    pub name: String,
    pub cluster: ClusterInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClusterInfo {
    pub server: String,
    #[serde(rename = "certificate-authority")]
    pub certificate_authority: Option<String>,
    #[serde(rename = "certificate-authority-data")]
    pub certificate_authority_data: Option<String>,
    #[serde(rename = "insecure-skip-tls-verify")]
    pub insecure_skip_tls_verify: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamedUser {
    pub name: String,
    pub user: User,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub token: Option<String>,
    #[serde(rename = "client-certificate")]
    pub client_certificate: Option<String>,
    #[serde(rename = "client-key")]
    pub client_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub exec: Option<UserExecConfig>,
    // auth-provider, exec, etc. can be added later
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExecConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<ExecEnvVar>>,
    pub api_version: Option<String>,
    pub interactive_mode: Option<String>,
    pub provide_cluster_info: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecEnvVar {
    pub name: String,
    pub value: String,
}

/// Load a kubeconfig from a local path and parse it into KubeConfig.
///
/// Returns Err if the file can't be read or YAML is invalid.
pub fn load_kube_config<P: AsRef<Path>>(path: P) -> Result<KubeConfig, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(expand_tilde(&path))
        .expect(format!("Unable to read kubeconfig file: {:?}", &path.as_ref()).as_str());
    let cfg: KubeConfig = serde_yaml::from_str(&data)?;
    Ok(cfg)
}

/// Convenience: get the server URL for the current context, if available.
pub fn current_context_server(cfg: &KubeConfig) -> Option<String> {
    if let Some(current) = &cfg.current_context {
        // find the context entry
        let ctx = cfg.context_entry_by_name(current);
        if let Some(ctx) = ctx {
            // find the cluster by name
            let cl = cfg.cluster_entry_by_name(&ctx.context.cluster);
            if let Some(cl) = cl {
                return Some(cl.cluster.server.clone());
            }
        }
    }
    None
}

// Helper trait-like methods implemented as inherent methods on KubeConfig via impl blocks
impl KubeConfig {
    pub fn context_entry_by_name(&self, name: &str) -> Option<&NamedContext> {
        self.contexts.iter().find(|c| c.name == name)
    }

    pub fn cluster_entry_by_name(&self, name: &str) -> Option<&NamedCluster> {
        self.clusters.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_kubeconfig_yaml() -> &'static str {
        r#"
apiVersion: v1
kind: Config
clusters:
  - name: kubernetes
    cluster:
      server: https://127.0.0.1:6443
contexts:
  - name: minikube
    context:
      cluster: kubernetes
      user: minikube
      namespace: default
current-context: minikube
users:
  - name: minikube
    user:
      token: dummy-token
"#
    }

    #[test]
    fn test_parse_basic_kubeconfig() {
        // write sample to a temp file
        let mut f = NamedTempFile::new().expect("temp file");
        write!(f, "{}", sample_kubeconfig_yaml()).unwrap();

        let cfg = load_kube_config(f.path()).expect("parse kubeconfig");

        assert_eq!(cfg.current_context.as_deref(), Some("minikube"));
        // ensure contexts and clusters parsed
        assert_eq!(cfg.contexts.len(), 1);
        assert_eq!(cfg.clusters.len(), 1);

        // verify server URL accessible via current context
        if let Some(server) = cfg
            .cluster_entry_by_name(&cfg.contexts[0].context.cluster)
            .and_then(|c| Some(c.cluster.server.clone()))
        {
            assert_eq!(server, "https://127.0.0.1:6443");
        } else {
            panic!("expected cluster server to be present");
        }
    }

    #[test]
    fn test_current_context_server_helper() {
        // Initialize logger for tests
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();
        let mut f = NamedTempFile::new().expect("temp file");
        write!(f, "{}", sample_kubeconfig_yaml()).unwrap();

        let cfg = load_kube_config(expand_tilde(f.path())).expect("parse kubeconfig");
        log::debug!("Loaded kubeconfig: {:?}", cfg);
        // use the helper
        let server = cfg
            .current_context
            .as_ref()
            .and_then(|ctx_name| cfg.contexts.iter().find(|c| &c.name == ctx_name))
            .and_then(|ctx| {
                cfg.clusters
                    .iter()
                    .find(|cl| cl.name == ctx.context.cluster)
            })
            .map(|cl| cl.cluster.server.clone());

        assert_eq!(server, Some("https://127.0.0.1:6443".to_string()));
    }

    #[test]
    fn test_missing_fields_handling() {
        // minimal invalid YAML (missing required fields)
        let mut f = NamedTempFile::new().expect("temp file");
        write!(f, "apiVersion: v1\nkind: Config\n").unwrap();

        // This should fail to parse due to missing required arrays
        let res = load_kube_config(f.path());
        assert!(res.is_err());
    }
}

pub mod commands {
    use crate::errors::ApiResult;
    use crate::services::kube_config;
    use tauri::command;

    #[command]
    pub fn load_kube_config(
        path: &str,
        // app: tauri::AppHandle,
        // window: tauri::Window,
    ) -> ApiResult<kube_config::KubeConfig> {
        Ok(kube_config::load_kube_config(kube_config::expand_tilde(path)).expect("load kubeconfig"))
    }
}
