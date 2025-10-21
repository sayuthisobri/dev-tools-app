#![allow(unused)]

use crate::errors::ApiResult;
use crate::http_request;
use crate::services::aws_s3::{PageableList, S3Bucket, S3Object};
use crate::services::kube_config::expand_tilde;
use crate::services::{aws, kube_config};
use tauri::command;


#[command]
pub fn gen_time(time: &str) -> String {
    format!("Time generated: {}", time)
}

#[command(async)]
pub async fn load_kube_config(
    path: &str,
    app: tauri::AppHandle,
    window: tauri::Window,
) -> ApiResult<kube_config::KubeConfig> {
    Ok(kube_config::load_kubeconfig(kube_config::expand_tilde(path)).expect("load kubeconfig"))
}

#[command(async)]
pub async fn aws_profiles(path: &str) -> ApiResult<Vec<String>> {
    Ok(aws::profiles_from_file(&expand_tilde(path)).await)
}

#[command(async)]
pub async fn http_send_request(
    req: http_request::HTTPRequest,
    timeout: Option<http_request::RequestTimeout>,
) -> ApiResult<http_request::HTTPResponse> {
    dbg!(req.clone());
    Ok(http_request::request(req, timeout).await?)
}

#[command(async)]
pub async fn aws_s3_buckets(profile: &str) -> ApiResult<PageableList<S3Bucket>> {
    let client = aws::AwsClient::get(profile).await?;
    Ok(client.list_buckets().await?)
}

#[command(async)]
pub async fn aws_s3_objects(profile: &str, bucket: &str) -> ApiResult<PageableList<S3Object>> {
    let client = aws::AwsClient::get(profile).await?;
    Ok(client.list_objects(bucket).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_one() {
        let result = gen_time("AA!").to_string();
        assert!(result.ends_with("AA!"));
    }
}
