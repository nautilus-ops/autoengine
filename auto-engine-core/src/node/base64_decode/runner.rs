use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Base64DecodeParams {
    pub data: String,
    #[serde(default)]
    pub url_safe: bool,
    #[serde(default)]
    pub no_padding: bool,
}

#[derive(Default)]
pub struct Base64DecodeRunner;

impl Base64DecodeRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn decode_bytes(&self, data: &str, url_safe: bool, no_padding: bool) -> Result<Vec<u8>, String> {
        let input = data.trim();
        match (url_safe, no_padding) {
            (false, false) => STANDARD.decode(input),
            (false, true) => STANDARD_NO_PAD.decode(input),
            (true, false) => URL_SAFE.decode(input),
            (true, true) => URL_SAFE_NO_PAD.decode(input),
        }
        .map_err(|e| format!("Failed to decode base64: {}", e))
    }
}

#[async_trait::async_trait]
impl NodeRunner for Base64DecodeRunner {
    type ParamType = Base64DecodeParams;

    async fn run(
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let decoded_bytes = self.decode_bytes(&params.data, params.url_safe, params.no_padding)?;

        let is_utf8 = String::from_utf8(decoded_bytes.clone()).is_ok();
        let decoded = String::from_utf8_lossy(&decoded_bytes).to_string();

        let mut result = HashMap::new();
        result.insert("decoded".to_string(), serde_json::json!(decoded));
        result.insert("is_utf8".to_string(), serde_json::json!(is_utf8));
        result.insert(
            "byte_length".to_string(),
            serde_json::json!(decoded_bytes.len() as i64),
        );

        Ok(Some(result))
    }
}

pub struct Base64DecodeRunnerFactory;

impl Base64DecodeRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Base64DecodeRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for Base64DecodeRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(Base64DecodeRunner::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn decode_standard() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = Base64DecodeRunner::new();
        let params = Base64DecodeParams {
            data: "aGVsbG8gd29ybGQ=".to_string(),
            url_safe: false,
            no_padding: false,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(
            result.get("decoded"),
            Some(&serde_json::json!("hello world"))
        );
        assert_eq!(result.get("is_utf8"), Some(&serde_json::json!(true)));
        assert_eq!(result.get("byte_length"), Some(&serde_json::json!(11)));
    }

    #[tokio::test]
    async fn decode_url_safe_binary() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = Base64DecodeRunner::new();
        let params = Base64DecodeParams {
            data: "__8".to_string(), // URL-safe base64 for 0xff 0xff without padding
            url_safe: true,
            no_padding: true,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("is_utf8"), Some(&serde_json::json!(false)));
        assert_eq!(result.get("byte_length"), Some(&serde_json::json!(2)));
        // Lossy conversion keeps length while surfacing replacement chars.
        assert_eq!(result.get("decoded"), Some(&serde_json::json!("��")));
    }
}
