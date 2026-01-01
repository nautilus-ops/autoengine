use crate::context::Context;
use once_cell::sync::Lazy;
use regex::Regex;

pub static REGEX_PARSE_VARIABLES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([^}:]+(?:\.[^}:]+)*)(?::([^}]*))?}").unwrap());

// String: the value name or key
// bool: if need get value from Context
pub async fn parse_variables(context: &Context, input: &str) -> String {
    let ctx = context.value.read().await;

    REGEX_PARSE_VARIABLES
        .replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            if let Some(value) = ctx.get(var_name) {
                return serde_json::to_string(&value.value)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();
            }
            default.to_string()
        })
        .into_owned()
}

pub async fn try_parse_variables(context: &Context, input: &str) -> Result<String, String> {
    let ctx = context.value.read().await;
    let mut err: Option<String> = None;

    let result = REGEX_PARSE_VARIABLES.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];

        let variable = match ctx.get(var_name) {
            Some(value) => serde_json::to_string(&value.value)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            None => {
                err = Some(format!("variable `{}` not found", var_name));
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

        for t in tests {
            let result = parse_variables(&context, &t.content).await;
            assert_eq!(t.expected, result);
        }
    }
}
