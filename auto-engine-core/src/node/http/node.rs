use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "HTTPClient";

#[derive(Default)]
pub struct HttpNode;

impl HttpNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for HttpNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "HTTP 请求".to_string(),
            en: "HTTP Request".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgY2xhc3M9Imx1Y2lkZSBsdWNpZGUtZ2xvYmUtaWNvbiBsdWNpZGUtZ2xvYmUiPjxjaXJjbGUgY3g9IjEyIiBjeT0iMTIiIHI9IjEwIi8+PHBhdGggZD0iTTEyIDJhMTQuNSAxNC41IDAgMCAwIDAgMjAgMTQuNSAxNC41IDAgMCAwIDAtMjAiLz48cGF0aCBkPSJNMiAxMmgyMCIvPjwvc3ZnPg==",
        )
    }

    fn category(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "网络请求".to_string(),
            en: "Network".to_string(),
        })
    }

    fn description(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "发送简单的 HTTP 请求（基于 reqwest）。".to_string(),
            en: "Send simple HTTP requests using reqwest.".to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "status".to_string(),
                field_type: FieldType::Number,
                item_type: None,
                description: Some(I18nValue {
                    zh: "HTTP 状态码".to_string(),
                    en: "HTTP status code".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "body".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "响应正文".to_string(),
                    en: "Response body".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
        ]
    }

    fn input_schema(&self) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "method".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "HTTP 方法，支持 GET/POST".to_string(),
                    en: "HTTP method, supports GET/POST".to_string(),
                }),
                enums: vec!["GET".to_string(), "POST".to_string()],
                default: Some("GET".to_string()),
                condition: None,
            },
            SchemaField {
                name: "url".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "请求 URL".to_string(),
                    en: "Request URL".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "headers".to_string(),
                field_type: FieldType::Array,
                item_type: Some(FieldType::String),
                description: Some(I18nValue {
                    zh: "可选请求头列表，格式：Key: Value 或 Key=Value".to_string(),
                    en: "Optional headers list, format: Key: Value or Key=Value".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "body".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "POST 请求体，文本/JSON 皆可。".to_string(),
                    en: "POST body, plain text or JSON string.".to_string(),
                }),
                enums: vec![],
                default: Some("".to_string()),
                condition: None,
            },
            SchemaField {
                name: "timeout_ms".to_string(),
                field_type: FieldType::Number,
                item_type: None,
                description: Some(I18nValue {
                    zh: "请求超时时间（毫秒），默认 30000。".to_string(),
                    en: "Request timeout in milliseconds, default 30000.".to_string(),
                }),
                enums: vec![],
                default: Some("30000".to_string()),
                condition: None,
            },
        ]
    }
}
