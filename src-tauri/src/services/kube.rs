use crate::errors::kube_error::KubeResult;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Pod;
use kube::core::ObjectList;
use kube::Api;
use log::{debug, trace};

pub struct KubeClient {
    client: kube::Client,
}

impl KubeClient {
    pub async fn new(context: Option<String>) -> KubeResult<KubeClient> {
        let client = get_client(context).await?;
        Ok(KubeClient { client })
    }

    async fn get_pods(&self, ns: &str) -> KubeResult<ObjectList<Pod>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), ns);
        Ok(pods.list(&Default::default()).await?)
    }

    async fn get_deployments(&self, ns: &str) -> KubeResult<ObjectList<Deployment>> {
        let res: Api<Deployment> = Api::namespaced(self.client.clone(), ns);

        Ok(res.list(&Default::default()).await?)
    }
}

async fn get_client(context: Option<String>) -> KubeResult<kube::Client> {
    debug!("env KUBECONFIG: {:?}", std::env::var_os("KUBECONFIG"));
    let client_config = match context.as_ref() {
        Some(context) => {
            debug!("Getting kubernetes client [{}]", context);
            kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions {
                context: Some(context.to_owned()),
                ..Default::default()
            })
            .await?
        }
        None => {
            debug!("Getting kubernetes client [default]");
            // kube::Config::infer().await?
            kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions {
                context: None,
                ..Default::default()
            })
            .await?
        }
    };
    trace!("Kubernetes client config: {:?}", client_config);
    Ok(kube::Client::try_from(client_config)?)
}

#[cfg(test)]
mod kube_test {
    use crate::services::kube::KubeClient;
    use crate::utils;
    use k8s_openapi::api::apps::v1::Deployment;
    use kube::{Api, ResourceExt};
    use log::LevelFilter::*;
    use log::*;

    fn init_logger(level: Option<LevelFilter>) {
        utils::test::init_test_logger(level.or(Some(Debug)).expect("no debug log level set"));
    }

    #[tokio::test]
    async fn test_get_client() {
        // Initialize logger for tests
        init_logger(Some(Debug));

        let res = KubeClient::new(Some("cdx-dev2".to_string())).await;
        match res {
            Ok(client) => {
                assert_eq!(
                    client
                        .client
                        .apiserver_version()
                        .await
                        .expect("Failed to get api server version")
                        .major
                        .len(),
                    1
                )
            }
            Err(err) => {
                debug!("err {:?}", err)
            }
        }
    }

    #[tokio::test]
    async fn test_get_pods() {
        init_logger(None);
        let k8s = KubeClient::new(Some(String::from("cdx-dev2")))
            .await
            .unwrap();
        let pods = k8s
            .get_pods(k8s.client.default_namespace())
            .await
            .expect("get_pods failed");

        debug!(
            "pods: {:?}",
            pods.items.iter().map(|p| p.name_any()).collect::<Vec<_>>()
        );
        assert!(!pods.items.is_empty());

        let deployments = k8s
            .get_deployments(k8s.client.default_namespace())
            .await
            .expect("get_deployments failed");

        debug!(
            "deployments: {:?}",
            deployments
                .items
                .iter()
                .map(|p| p.name_any())
                .collect::<Vec<_>>()
        );
        assert!(!deployments.items.is_empty());
    }

    #[tokio::test]
    async fn test_get_scale() {
        init_logger(None);
        let k8s = KubeClient::new(Some(String::from("cdx-dev2")))
            .await
            .unwrap();
        let res: Api<Deployment> =
            Api::namespaced(k8s.client.clone(), k8s.client.default_namespace());
        let scale = res
            .get_scale("cdx-transaction")
            .await
            .expect("get_scale failed");

        print!(
            "{}",
            serde_yaml::to_string(&scale).expect("Failed to serialize")
        );
    }
}
