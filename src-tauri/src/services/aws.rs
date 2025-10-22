use crate::errors::aws_error::AwsResult;
use crate::errors::AwsError;
use crate::services::aws_s3::S3Client;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AwsClient {
    pub(in crate::services) s3: Option<S3Client>,
    pub(in crate::services) profile: String,
}

static AWS_SESSION: Lazy<Mutex<Option<HashMap<String, AwsClient>>>> =
    Lazy::new(|| Mutex::new(Some(HashMap::new())));

pub async fn remove_profile(profile: &str) -> Option<AwsClient> {
    let mut aws_session = AWS_SESSION.lock().await;
    let aws_session = aws_session.as_mut().expect("Unable to load AWS sessions");
    aws_session.remove(profile)
}

impl AwsClient {
    pub async fn get(profile: &str) -> AwsResult<AwsClient> {
        Self::check_profile(&profile).await?;

        let mut aws_session = AWS_SESSION.lock().await;
        let aws_session = aws_session.as_mut().unwrap();
        if aws_session.contains_key(profile) {
            return Ok(aws_session.get(profile).unwrap().clone());
        }

        let client = AwsClient {
            s3: None,
            profile: profile.to_string(),
        };
        aws_session.insert(profile.to_string(), client.clone());
        Ok(client)
    }

    async fn check_profile(profile: &&str) -> AwsResult<()> {
        let sts = Command::new("aws")
            .args(&[
                "sts",
                "get-caller-identity",
                "--profile",
                &profile,
                "--output",
                "json",
            ])
            .output()
            .expect(format!("Failed to identify profile {:?}", profile).as_str());
        if !sts.status.success() {
            let err = String::from_utf8_lossy(&sts.stderr);
            if err.contains("SSO Token") && err.contains("does not exist") {
                Command::new("aws")
                    .args(&["sso", "login", "--profile", &profile])
                    .status()
                    .expect("Failed to login");
            }
            // eprintln!("aws sts get-caller-identity failed: {}", err);
            remove_profile(profile).await;
            return Err(AwsError::AwsProfile(
                profile.to_string(),
                err.trim().to_string(),
            ));
        }
        Ok(())
    }
}

pub async fn profiles_from_file(path: &PathBuf) -> Vec<String> {
    // let default_path = expand_tilde("~/.aws/config");
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![], // ignore missing files gracefully
    };

    // Collect section names like [default], [profile dev], [my-profile]
    let mut profiles = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') && line.len() > 2 {
            // strip brackets
            let inner = &line[1..line.len() - 1];
            // In config, sections can be "default" or "profile NAME"
            let prof = if inner.starts_with("profile ") {
                inner[8..].to_string()
            } else {
                inner.to_string()
            };
            if !prof.is_empty() {
                profiles.push(prof);
            }
        }
    }
    profiles
}

pub mod commands {
    use crate::errors::ApiResult;
    use crate::services::aws;
    use crate::utils::expand_tilde;
    use tauri::command;

    #[command(async)]
    pub async fn aws_profiles(path: &str) -> ApiResult<Vec<String>> {
        Ok(aws::profiles_from_file(&expand_tilde(path)).await)
    }
}

#[cfg(test)]
mod test {
    use crate::errors::aws_error::AwsResult;
    use crate::services::aws::{profiles_from_file, AwsClient};
    use crate::utils::{expand_tilde, init_test_logger};
    use log::LevelFilter::Debug;
    use std::path::PathBuf;
    use tracing::debug;

    #[tokio::test]
    async fn test_get_s3_buckets() -> AwsResult<()> {
        init_test_logger(Debug);
        let mut client = AwsClient::get("CloudEngineer-411632713503").await?;
        // let mut client = AwsClient::get("reldyn").await?;
        client.prepare_s3().await;
        // let client = AwsClient::new("finodyn").await;
        let res = client.list_buckets().await.expect("list buckets");
        debug!(target: "s3-bucket", "\n{}", serde_yaml::to_string(&res).unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_s3_objects() {
        // let client = AwsClient::new("CloudEngineer-411632713503").await;
        let client = AwsClient::get("finodyn")
            .await
            .expect("failed to create client");
        let res = client
            .list_objects("cdx-banking-dev02-settlement")
            .await
            .expect("list buckets");
        println!("{}", serde_yaml::to_string(&res).unwrap());
    }

    #[tokio::test]
    async fn test_profiles_from_file() {
        let profiles = profiles_from_file(&PathBuf::from(expand_tilde("~/.aws/config"))).await;
        println!("{:?}", profiles);
    }
}
