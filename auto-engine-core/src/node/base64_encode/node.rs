use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "Base64Encode";

#[derive(Default)]
pub struct Base64EncodeNode;

impl Base64EncodeNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for Base64EncodeNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "Base64 编码".to_string(),
            en: "Base64 Encode".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHJlY3QgeD0iMyIgeT0iMyIgd2lkdGg9IjE4IiBoZWlnaHQ9IjE4IiByeD0iMiIvPjxwYXRoIGQ9Ik04IDE0aDgiLz48cGF0aCBkPSJNMTIgMTBWNiIvPjxwYXRoIGQ9Ik05LjUgOC41IDEyIDZsMi41IDIuNSIvPjwvc3ZnPg==",
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
            zh: "将文本编码为 Base64，支持 URL Safe 与去填充配置。".to_string(),
            en: "Encode text into Base64 with optional URL-safe alphabet and padding control."
                .to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "encoded".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "编码后的 Base64 字符串".to_string(),
                    en: "Base64-encoded string.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "byte_length".to_string(),
                field_type: FieldType::Number,
                item_type: None,
                description: Some(I18nValue {
                    zh: "原始字符串的字节长度".to_string(),
                    en: "Byte length of the input string.".to_string(),
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
                name: "data".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "需要编码的原始文本".to_string(),
                    en: "Raw text to encode.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "url_safe".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "是否使用 URL Safe 字母表".to_string(),
                    en: "Use the URL-safe alphabet.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "no_padding".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "是否移除末尾的填充符号 \"=\"".to_string(),
                    en: "Remove trailing padding characters (=).".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
        ]
    }
}
