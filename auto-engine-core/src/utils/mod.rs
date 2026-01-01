use crate::context::{Context, ValueItem};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

pub static REGEX_PARSE_VARIABLES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([^}:]+(?:\.[^}:]+)*)(?::([^}]*))?}").unwrap());

fn stringify_value(value: &serde_json::Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn resolve_variable_value(
    ctx: &HashMap<String, ValueItem>,
    var_name: &str,
) -> Result<Option<String>, String> {
    let parts: Vec<&str> = var_name.split('.').collect();

    if parts.len() >= 3 && parts[0] == "ctx" {
        let ctx_key = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        let mut value = match ctx.get(&ctx_key) {
            Some(item) => item.value.clone(),
            None => return Ok(None),
        };

        for key in parts.iter().skip(3) {
            match value {
                serde_json::Value::Object(map) => {
                    if let Some(next_value) = map.get(*key) {
                        value = next_value.clone();
                    } else {
                        return Err(format!("variable `{}` not found", var_name));
                    }
                }
                _ => {
                    return Err(format!(
                        "variable `{}` is not an object, cannot access `{}`",
                        ctx_key, key
                    ))
                }
            }
        }

        return Ok(Some(stringify_value(&value)));
    }

    Ok(ctx.get(var_name).map(|value| stringify_value(&value.value)))
}

// String: the value name or key
// bool: if need get value from Context
pub async fn parse_variables(context: &Context, input: &str) -> String {
    let ctx = context.value.read().await;

    REGEX_PARSE_VARIABLES
        .replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            match resolve_variable_value(&ctx, var_name) {
                Ok(Some(value)) => value,
                _ => default.to_string(),
            }
        })
        .into_owned()
}

pub async fn try_parse_variables(context: &Context, input: &str) -> Result<String, String> {
    let ctx = context.value.read().await;
    let mut err: Option<String> = None;

    let result = REGEX_PARSE_VARIABLES.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];

        let variable = match resolve_variable_value(&ctx, var_name) {
            Ok(Some(value)) => value,
            Ok(None) => {
                err = Some(format!("variable `{}` not found", var_name));
                String::new()
            }
            Err(e) => {
                err = Some(e);
                String::new()
            }
        };
        if variable == "" {
            err = Some(format!("variable `{}` is empty", var_name));
        }
        variable
    });

    if let Some(e) = err {
        Err(e)
    } else {
        Ok(result.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_parse_variables() {
        struct TestCase {
            pub content: String,
            pub expected: String,
        }

        let tests: Vec<TestCase> = vec![
            TestCase {
                content: "${test:a}".to_string(),
                expected: "test_value".to_string(),
            },
            TestCase {
                content: "${none.a}".to_string(),
                expected: "a".to_string(),
            },
            TestCase {
                content: "b".to_string(),
                expected: "b".to_string(),
            },
            TestCase {
                content: "${image-rec.x:0}".to_string(),
                expected: "0".to_string(),
            },
            TestCase {
                content: "${image-rec.x:0} > 2".to_string(),
                expected: "0 > 2".to_string(),
            },
            TestCase {
                content: "${ctx.node.output.sub.value}".to_string(),
                expected: "nested".to_string(),
            },
        ];

        #[cfg(feature = "tauri")]
        let context = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let context = Context::new(PathBuf::new());

        context
            .set_string_value("test", "test_value")
            .await
            .unwrap();
        context.set_string_value("none.a", "a").await.unwrap();
        context
            .set_value(
                "ctx.node.output",
                json!({"sub": {"value": "nested"}}),
                String::new(),
            )
            .await
            .unwrap();

        for t in tests {
            let result = parse_variables(&context, &t.content).await;
            assert_eq!(t.expected, result);
        }
    }

    #[tokio::test]
    async fn test_try_parse_variables_nested_error() {
        #[cfg(feature = "tauri")]
        let context = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let context = Context::new(PathBuf::new());

        context
            .set_value("ctx.node.output", json!({"sub": 1}), String::new())
            .await
            .unwrap();

        let result = try_parse_variables(&context, "${ctx.node.output.missing}").await;
        assert!(result.is_err());
    }
}
