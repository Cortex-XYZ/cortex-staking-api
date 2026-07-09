use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum MonadClientError {
    RpcRequestFailed,
    RpcError(String),
    InvalidResponse,
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
            .ok_or(MonadClientError::InvalidResponse)
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
            .ok_or(MonadClientError::InvalidResponse)
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
            .map_err(|_| MonadClientError::RpcRequestFailed)?;

        let body = response
            .json::<JsonRpcResponse>()
            .await
            .map_err(|_| MonadClientError::InvalidResponse)?;

        if let Some(error) = body.error {
            return Err(MonadClientError::RpcError(error.message));
        }

        body.result.ok_or(MonadClientError::InvalidResponse)
    }
}