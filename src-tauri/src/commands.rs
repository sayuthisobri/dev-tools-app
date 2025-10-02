#![allow(unused)]

use crate::error::APIError;
use crate::http_request;
use anyhow::Result;
use tauri::command;

pub type CommandResult<T> = Result<T, APIError>;

#[command]
pub fn gen_time(time: &str) -> String {
    format!("Time generated: {}", time)
}

#[command(async)]
pub async fn http_request(
    req: http_request::HTTPRequest,
    timeout: Option<http_request::RequestTimeout>,
) -> CommandResult<http_request::HTTPResponse> {
    dbg!(req.clone());
    Ok(http_request::request(req, timeout).await?)
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
