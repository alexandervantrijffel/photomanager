use std::sync::Arc;

use anyhow::Result;
use hyper::HeaderMap;

pub async fn post_json(
    url: &str,
    headers: HeaderMap,
    client: Arc<reqwest::Client>,
    json: &serde_json::Value,
) -> Result<ReqwestResult> {
    let response = client.post(url).headers(headers).json(json).send().await?;

    let status = &response.status();
    let response_body = &response.text().await?;
    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Failed to post json to {}. Status: {}. Response body: {}",
            url,
            status,
            response_body
        ));
    }

    Ok(ReqwestResult {
        status: *status,
        response_body: response_body.clone(),
    })
}

pub struct ReqwestResult {
    pub status: reqwest::StatusCode,
    pub response_body: String,
}
