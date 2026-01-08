use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseMode {
    Lowercase,
    Uppercase,
    Title,
    Capitalize,
    Swapcase,
}

impl CaseMode {
    fn as_str(&self) -> &'static str {
        match self {
            CaseMode::Lowercase => "lowercase",
            CaseMode::Uppercase => "uppercase",
            CaseMode::Title => "title",
            CaseMode::Capitalize => "capitalize",
            CaseMode::Swapcase => "swapcase",
        }
    }

    fn apply(&self, text: &str) -> String {
        match self {
            CaseMode::Lowercase => text.to_lowercase(),
            CaseMode::Uppercase => text.to_uppercase(),
            CaseMode::Title => title_case(text),
            CaseMode::Capitalize => capitalize_sentence(text),
            CaseMode::Swapcase => swap_case(text),
        }
    }
}

impl Default for CaseMode {
    fn default() -> Self {
        CaseMode::Lowercase
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextCaseConvertParams {
    pub text: String,
    #[serde(default)]
    pub mode: CaseMode,
}

#[derive(Default)]
pub struct TextCaseConvertRunner;

impl TextCaseConvertRunner {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl NodeRunner for TextCaseConvertRunner {
    type ParamType = TextCaseConvertParams;

    async fn run(
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let result = params.mode.apply(&params.text);

        let mut map = HashMap::new();
        map.insert("result".to_string(), serde_json::json!(result));
        map.insert("mode".to_string(), serde_json::json!(params.mode.as_str()));

        Ok(Some(map))
    }
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => {
            let mut result = String::new();
            result.extend(first.to_uppercase());
            result.push_str(&chars.as_str().to_lowercase());
            result
        }
        None => String::new(),
    }
}

fn title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut current_word = String::new();

    for ch in text.chars() {
        if ch.is_whitespace() {
            result.push_str(&capitalize_word(&current_word));
            current_word.clear();
            result.push(ch);
        } else {
            current_word.push(ch);
        }
    }

    if !current_word.is_empty() {
        result.push_str(&capitalize_word(&current_word));
    }

    result
}

fn capitalize_sentence(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => {
            let mut result = String::new();
            result.extend(first.to_uppercase());
            result.push_str(&chars.as_str().to_lowercase());
            result
        }
        None => String::new(),
    }
}

fn swap_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_uppercase() {
            result.extend(ch.to_lowercase());
        } else if ch.is_lowercase() {
            result.extend(ch.to_uppercase());
        } else {
            result.push(ch);
        }
    }
    result
}

pub struct TextCaseConvertRunnerFactory;

impl TextCaseConvertRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TextCaseConvertRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for TextCaseConvertRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(TextCaseConvertRunner::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn converts_to_uppercase() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextCaseConvertRunner::new();
        let params = TextCaseConvertParams {
            text: "Hello World".to_string(),
            mode: CaseMode::Uppercase,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("HELLO WORLD")));
        assert_eq!(result.get("mode"), Some(&serde_json::json!("uppercase")));
    }

    #[tokio::test]
    async fn converts_to_title_case() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextCaseConvertRunner::new();
        let params = TextCaseConvertParams {
            text: "hello   rust\nrocks".to_string(),
            mode: CaseMode::Title,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(
            result.get("result"),
            Some(&serde_json::json!("Hello   Rust\nRocks"))
        );
        assert_eq!(result.get("mode"), Some(&serde_json::json!("title")));
    }

    #[tokio::test]
    async fn swaps_case() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextCaseConvertRunner::new();
        let params = TextCaseConvertParams {
            text: "Rust123".to_string(),
            mode: CaseMode::Swapcase,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("rUST123")));
        assert_eq!(result.get("mode"), Some(&serde_json::json!("swapcase")));
    }
}
