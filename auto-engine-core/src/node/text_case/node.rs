use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "TextCaseConvert";

#[derive(Default)]
pub struct TextCaseConvertNode;

impl TextCaseConvertNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for TextCaseConvertNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "字符串大小写转换".to_string(),
            en: "Text Case Convert".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFsYXJnZS1zbWFsbC1pY29uIGx1Y2lkZS1hLWxhcmdlLXNtYWxsIj48cGF0aCBkPSJtMTUgMTYgMi41MzYtNy4zMjhhMS4wMiAxLjAyIDEgMCAxIDEuOTI4IDBMMjIgMTYiLz48cGF0aCBkPSJNMTUuNjk3IDE0aDUuNjA2Ii8+PHBhdGggZD0ibTIgMTYgNC4wMzktOS42OWEuNS41IDAgMCAxIC45MjMgMEwxMSAxNiIvPjxwYXRoIGQ9Ik0zLjMwNCAxM2g2LjM5MiIvPjwvc3ZnPg==",
        )
    }

    fn category(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "文本处理".to_string(),
            en: "Text Processing".to_string(),
        })
    }

    fn description(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "将文本转换为大写、小写、标题格式或大小写反转。".to_string(),
            en: "Convert text into upper case, lower case, title case, or swap its casing."
                .to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "result".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "转换后的文本".to_string(),
                    en: "Transformed text.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "mode".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "实际使用的转换模式".to_string(),
                    en: "Mode applied during conversion.".to_string(),
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
                name: "text".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "需要转换大小写的文本内容".to_string(),
                    en: "Text content to convert.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "mode".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "转换模式：lowercase / uppercase / title / capitalize / swapcase".to_string(),
                    en: "Conversion mode: lowercase / uppercase / title / capitalize / swapcase"
                        .to_string(),
                }),
                enums: vec![
                    "lowercase".to_string(),
                    "uppercase".to_string(),
                    "title".to_string(),
                    "capitalize".to_string(),
                    "swapcase".to_string(),
                ],
                default: Some("lowercase".to_string()),
                condition: None,
            },
        ]
    }
}
