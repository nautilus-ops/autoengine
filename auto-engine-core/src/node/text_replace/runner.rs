use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextReplaceParams {
    pub source: String,
    pub pattern: String,
    #[serde(default)]
    pub replacement: String,
    #[serde(default)]
    pub use_regex: bool,
    #[serde(default)]
    pub case_insensitive: bool,
    #[serde(default = "TextReplaceParams::default_replace_all")]
    pub replace_all: bool,
}

impl TextReplaceParams {
    fn default_replace_all() -> bool {
        true
    }
}

#[derive(Default)]
pub struct TextReplaceRunner;

impl TextReplaceRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn replace_with_regex(
        &self,
        source: &str,
        pattern: &str,
        replacement: &str,
        case_insensitive: bool,
        replace_all: bool,
    ) -> Result<(String, u64), String> {
        let pattern = if case_insensitive {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        let regex =
            Regex::new(&pattern).map_err(|e| format!("Invalid regular expression: {}", e))?;

        if replace_all {
            let count = regex.find_iter(source).count() as u64;
            let replaced = regex
                .replace_all(source, replacement)
                .into_owned();
            Ok((replaced, count))
        } else if let Some(mat) = regex.find(source) {
            let mut result = source.to_string();
            result.replace_range(mat.range(), replacement);
            Ok((result, 1))
        } else {
            Ok((source.to_string(), 0))
        }
    }

    fn replace_plain(
        &self,
        source: &str,
        pattern: &str,
        replacement: &str,
        replace_all: bool,
    ) -> Result<(String, u64), String> {
        if pattern.is_empty() {
            return Err("pattern cannot be empty when regex is disabled".to_string());
        }

        if replace_all {
            let count = source.matches(pattern).count() as u64;
            Ok((source.replace(pattern, replacement), count))
        } else if let Some(pos) = source.find(pattern) {
            let mut result = source.to_string();
            result.replace_range(pos..pos + pattern.len(), replacement);
            Ok((result, 1))
        } else {
            Ok((source.to_string(), 0))
        }
    }
}

#[async_trait::async_trait]
impl NodeRunner for TextReplaceRunner {
    type ParamType = TextReplaceParams;

    async fn run (
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let (result, count) = if params.use_regex {
            self.replace_with_regex(
                &params.source,
                &params.pattern,
                &params.replacement,
                params.case_insensitive,
                params.replace_all,
            )?
        } else {
            self.replace_plain(
                &params.source,
                &params.pattern,
                &params.replacement,
                params.replace_all,
            )?
        };

        let mut map = HashMap::new();
        map.insert("result".to_string(), serde_json::json!(result));
        map.insert(
            "replaced_count".to_string(),
            serde_json::json!(count as i64),
        );

        Ok(Some(map))
    }
}

pub struct TextReplaceRunnerFactory;

impl TextReplaceRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TextReplaceRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for TextReplaceRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(TextReplaceRunner::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn replace_plain_text_all() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextReplaceRunner::new();
        let params = TextReplaceParams {
            source: "foo bar foo".to_string(),
            pattern: "foo".to_string(),
            replacement: "baz".to_string(),
            use_regex: false,
            case_insensitive: false,
            replace_all: true,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("baz bar baz")));
        assert_eq!(result.get("replaced_count"), Some(&serde_json::json!(2)));
    }

    #[tokio::test]
    async fn replace_regex_first_only() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextReplaceRunner::new();
        let params = TextReplaceParams {
            source: "Hello HELLO".to_string(),
            pattern: "hello".to_string(),
            replacement: "hi".to_string(),
            use_regex: true,
            case_insensitive: true,
            replace_all: false,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("hi HELLO")));
        assert_eq!(result.get("replaced_count"), Some(&serde_json::json!(1)));
    }
}
