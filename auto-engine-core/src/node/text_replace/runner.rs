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
    pub replacements_json: Option<String>,
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

    fn parse_replacements_map(
        &self,
        replacements_json: &Option<String>,
    ) -> Result<Vec<(String, String)>, String> {
        let json = match replacements_json {
            Some(val) if !val.trim().is_empty() => val,
            _ => return Ok(vec![]),
        };

        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| format!("Invalid replacements_json, must be valid JSON object: {}", e))?;

        let obj = value.as_object().ok_or_else(|| {
            "Invalid replacements_json, expected JSON object with string key-value pairs".to_string()
        })?;

        let mut replacements = Vec::new();
        for (k, v) in obj.iter() {
            let value_str = v.as_str().ok_or_else(|| {
                format!(
                    "Invalid replacements_json value for key '{}', expected string",
                    k
                )
            })?;
            replacements.push((k.clone(), value_str.to_string()));
        }

        Ok(replacements)
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

    fn replace_with_map(
        &self,
        source: &str,
        replacements: &[(String, String)],
        replace_all: bool,
    ) -> Result<(String, u64), String> {
        let mut result = source.to_string();
        let mut total = 0;

        log::info!("replacements: =====> {:?}", replacements);

        for (pattern, replacement) in replacements.iter() {
            let (next, count) = self.replace_plain(
                &result,
                pattern,
                replacement,
                replace_all,
            )?;
            result = next;
            total += count;
        }

        Ok((result, total))
    }
}

#[async_trait::async_trait]
impl NodeRunner for TextReplaceRunner {
    type ParamType = TextReplaceParams;

    async fn run(
        &mut self,
        _ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let replacements = self.parse_replacements_map(&params.replacements_json)?;

        log::info!("source ===> {}", params.source);

        let (result, count) = if !replacements.is_empty() {
            self.replace_with_map(&params.source, &replacements, params.replace_all)?
        } else if params.use_regex {
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

        log::info!("result ===> {}", serde_json::json!(result));

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
            replacements_json: None,
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
            replacements_json: None,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("hi HELLO")));
        assert_eq!(result.get("replaced_count"), Some(&serde_json::json!(1)));
    }

    #[tokio::test]
    async fn replace_with_json_map() {
        #[cfg(feature = "tauri")]
        let ctx = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let ctx = Context::new(PathBuf::new());

        let mut runner = TextReplaceRunner::new();
        let params = TextReplaceParams {
            source: "foo hello foo".to_string(),
            pattern: "".to_string(),
            replacement: "".to_string(),
            replacements_json: Some("{\"foo\":\"bar\",\"hello\":\"hi\"}".to_string()),
            use_regex: false,
            case_insensitive: false,
            replace_all: true,
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        assert_eq!(result.get("result"), Some(&serde_json::json!("bar hi bar")));
        assert_eq!(result.get("replaced_count"), Some(&serde_json::json!(3)));
    }
}
