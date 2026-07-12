use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum MonadClientError {
    RpcRequestFailed(String),
    HttpError {
        status: u16,
        body: String,
    },
    RpcError {
        code: Option<i64>,
        message: String,
    },
    InvalidResponse(String),
}

#[derive(Clone)]
pub struct MonadRpcClient {
    http_client: Client,
    rpc_url: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    method: &'a str,
    params: Vec<Value>,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: Option<i64>,
    message: String,
}

impl MonadRpcClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            http_client: Client::new(),
            rpc_url: rpc_url.into(),
        }
    }

    pub async fn get_balance(
        &self,
        wallet_address: &str,
    ) -> Result<String, MonadClientError> {
        let result = self
            .rpc_call(
                "eth_getBalance",
                vec![
                    Value::String(wallet_address.to_string()),
                    Value::String("latest".to_string()),
                ],
            )
            .await?;

        result
            .as_str()
            .map(|value| value.to_string())
            .ok_or_else(|| {
                MonadClientError::InvalidResponse(
                    "JSON-RPC result was not a string".to_string(),
                )
            })
    }

    pub async fn eth_call(
        &self,
        to: &str,
        data: &str,
    ) -> Result<String, MonadClientError> {
        let result = self
            .rpc_call(
                "eth_call",
                vec![
                    serde_json::json!({
                        "to": to,
                        "data": data,
                    }),
                    Value::String("latest".to_string()),
                ],
            )
            .await?;

        result
            .as_str()
            .map(|value| value.to_string())
            .ok_or_else(|| {
                MonadClientError::InvalidResponse(
                    "JSON-RPC result was not a string".to_string(),
                )
            })
    }

    async fn rpc_call(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, MonadClientError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };

        let response = self
            .http_client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|error| {
                MonadClientError::RpcRequestFailed(error.to_string())
            })?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .map_err(|error| {
                MonadClientError::InvalidResponse(error.to_string())
            })?;

        if !status.is_success() {
            return Err(MonadClientError::HttpError {
                status: status.as_u16(),
                body: response_body,
            });
        }

        let body: JsonRpcResponse = serde_json::from_str(&response_body)
            .map_err(|error| {
                MonadClientError::InvalidResponse(format!(
                    "{}; response body: {}",
                    error, response_body
                ))
            })?;

        if let Some(error) = body.error {
            return Err(MonadClientError::RpcError {
                code: error.code,
                message: error.message,
            });
        }

        body.result.ok_or_else(|| {
            MonadClientError::InvalidResponse(
                "JSON-RPC response contained neither result nor error".to_string(),
            )
        })
    }
}