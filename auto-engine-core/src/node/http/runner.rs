use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpParams {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Option<Vec<String>>,
    #[serde(default)]
    pub body: String,
    #[serde(default = "HttpParams::default_timeout")]
    pub timeout_ms: u64,
}

impl HttpParams {
    fn default_timeout() -> u64 {
        30_000
    }
}

#[derive(Default, Clone)]
pub struct HttpRunner {
    client: Client,
}

impl HttpRunner {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn parse_headers(&self, headers: Option<Vec<String>>) -> Result<HeaderMap, String> {
        let mut map = HeaderMap::new();
        if let Some(list) = headers {
            for item in list {
                let parts: Vec<&str> = item.splitn(2, ':').collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Invalid header format '{}', expected 'Key: Value'",
                        item
                    ));
                }
                let name = parts[0].trim();
                let value = parts[1].trim();
                let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| format!("Invalid header name '{}': {}", name, e))?;
                let header_value = reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| format!("Invalid header value for '{}': {}", name, e))?;
                map.insert(header_name, header_value);
            }
        }
        Ok(map)
    }
}

#[async_trait::async_trait]
impl NodeRunner for HttpRunner {
    type ParamType = HttpParams;

    async fn run(
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let method = match params.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            other => return Err(format!("Unsupported method '{}'", other)),
        };

        let headers = self.parse_headers(params.headers)?;

        let request = self
            .client
            .request(method, &params.url)
            .headers(headers)
            .timeout(Duration::from_millis(params.timeout_ms));

        let request = if params.method.eq_ignore_ascii_case("POST") && !params.body.is_empty() {
            request.body(params.body)
        } else {
            request
        };

        let resp = request
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let body_obj: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|e| {
            log::warn!("Failed to parse response body as JSON: {}", e);
            serde_json::json!(body)
        });

        let mut res = HashMap::new();
        res.insert("status".to_string(), serde_json::json!(status));
        res.insert("body".to_string(), body_obj);
        Ok(Some(res))
    }
}

pub struct HttpRunnerFactory;

impl HttpRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HttpRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for HttpRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(HttpRunner::new()))
    }
}
