use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Base64EncodeParams {
    pub data: String,
    #[serde(default)]
    pub url_safe: bool,
    #[serde(default)]
    pub no_padding: bool,
}

#[derive(Default)]
pub struct Base64EncodeRunner;

impl Base64EncodeRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn encode(&self, data: &str, url_safe: bool, no_padding: bool) -> String {
        match (url_safe, no_padding) {
            (false, false) => STANDARD.encode(data),
            (false, true) => STANDARD_NO_PAD.encode(data),
            (true, false) => URL_SAFE.encode(data),
            (true, true) => URL_SAFE_NO_PAD.encode(data),
        }
    }
}

#[async_trait::async_trait]
impl NodeRunner for Base64EncodeRunner {
    type ParamType = Base64EncodeParams;

    async fn run(
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let encoded = self.encode(&params.data, params.url_safe, params.no_padding);

        let mut result = HashMap::new();
        result.insert("encoded".to_string(), serde_json::json!(encoded));
        result.insert(
            "byte_length".to_string(),
            serde_json::json!(params.data.as_bytes().len() as i64),
        );

        Ok(Some(result))
    }
}

pub struct Base64EncodeRunnerFactory;

impl Base64EncodeRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Base64EncodeRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for Base64EncodeRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(Base64EncodeRunner::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn encode_standard() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = Base64EncodeRunner::new();
        let params = Base64EncodeParams {
            data: "hello world".to_string(),
            url_safe: false,
            no_padding: false,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(
            result.get("encoded"),
            Some(&serde_json::json!("aGVsbG8gd29ybGQ="))
        );
        assert_eq!(result.get("byte_length"), Some(&serde_json::json!(11)));
    }

    #[tokio::test]
    async fn encode_url_safe_no_pad() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = Base64EncodeRunner::new();
        let params = Base64EncodeParams {
            data: "https://example.com/resource?id=1".to_string(),
            url_safe: true,
            no_padding: true,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(
            result.get("encoded"),
            Some(&serde_json::json!("aHR0cHM6Ly9leGFtcGxlLmNvbS9yZXNvdXJjZT9pZD0x"))
        );
        assert_eq!(result.get("byte_length"), Some(&serde_json::json!(33)));
    }
}
