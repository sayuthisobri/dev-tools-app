use crate::utils::*;
use bon::Builder;
use reqwest::redirect::Policy;
use reqwest::{ClientBuilder, Method, Url};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
pub struct Req {
    pub url: String,
    pub method: Option<String>,
    pub id: Option<String>,
    pub body: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
    pub timeout: Option<u64>,
    pub max_redirects: Option<u64>,
    pub allow_redirects: Option<bool>,
    pub user_agent: Option<String>,
    pub auth: Option<(String, String)>,
    pub cookies: Option<Vec<(String, String)>>,
    pub params: Option<Vec<(String, String)>>,
}

#[derive(Builder, Default, Debug, Serialize, Deserialize)]
pub struct Res {
    pub req: Req,
    pub url: String,
    pub status: u16,
    pub body: String,
    pub headers: Option<Vec<(String, String)>>,
    pub auth: Option<Vec<(String, String)>>,
    pub cookies: Option<Vec<(String, String)>>,
    pub params: Option<Vec<(String, String)>>,
}

pub async fn send_request(req: Req) -> Result<Res> {
    let orig: Req = req.clone();
    let client = ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(Policy::limited(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .build()?;

    let url: String;
    match req.params {
        Some(v) => {
            url = Url::parse_with_params(req.url.as_str(), v)
                .unwrap()
                .to_string()
        }
        None => {
            url = req.url.to_string();
        }
    }
    let req_builder = client.request(
        (Method::from_str(&req.method.unwrap_or("GET".to_string()))).unwrap_or(Method::GET),
        url,
    );
    // client.post(req.url)
    //     .json(&params) // Serialize params to JSON
    let response = req_builder.send().await?;

    // Check if the response was successful
    let res_builder = Res::builder()
        .req(orig)
        .status(response.status().as_u16())
        .url(response.url().to_string())
        .headers(
            response
                .headers()
                .iter()
                .map(|x| (x.0.to_string(), x.1.to_str().unwrap().to_string()))
                .collect(),
        )
        .cookies(
            response
                .cookies()
                .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()))
                .collect(),
        )
        .body(response.text().await?);
    Ok(res_builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_request() {
        let req = Req::builder()
            .url("https://httpbin.org/get".to_string())
            .build();
        let res = send_request(req).await.unwrap();
        assert_eq!(res.status, 200);
        println!("{:?}", res);
    }
}
