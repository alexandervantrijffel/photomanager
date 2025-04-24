use anyhow::{anyhow, Context, Result};
use hyper::HeaderMap;
use reqwest::Client;
use serde::Serialize;
use std::num::NonZeroU16;
use std::sync::Arc;
use tracing::{debug, error};

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

#[allow(dead_code)]
pub async fn get<T>(client: &Client, url: &str, headers: HeaderMap) -> Result<HttpResponse<T>>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    let response = client.get(url).headers(headers).send().await?;
    let status = &response.status();
    debug!(
        "Get reqwest {} status: {} Headers: {:#?}.",
        url,
        status,
        response.headers(),
    );
    let response_body = &response.text().await?;

    if !status.is_success() {
        let err = format!("Failed to get {url}. Status: {status}. Response body: {response_body}",);
        error!("{}", err);
        return Err(anyhow::anyhow!(err));
    }

    // debug!("response Body: {}", response_body);
    let data = serde_json::from_str::<T>(response_body)
        .with_context(|| {
            anyhow!(
                "Failed to unmarshal REST response to type {}",
                std::any::type_name::<T>()
            )
        })
        .map(|r| {
            debug!("Get reqwest received: {:?}", r);
            r
        })?;
    Ok(HttpResponse::<T> {
        succeeded: true,
        status: StatusCodeSerializable(NonZeroU16::new(status.as_u16()).unwrap()),
        data,
    })
    // let result = response
    //     .json::<BybitResponse<Position>>()
    //     .await
    //     .map_err(|e| anyhow::anyhow!("Error: {}", e));
}

#[derive(Debug, Serialize)]
// #[serde(remote = "StatusCode")]
pub struct StatusCodeSerializable(NonZeroU16);

#[derive(Debug, Serialize)]
pub struct HttpResponse<T>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    succeeded: bool,
    // #[serde(with = "StatusCodeSerializable")]
    // status: StatusCode,
    status: StatusCodeSerializable,
    data: T,
}
