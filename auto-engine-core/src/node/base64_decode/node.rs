use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "Base64Decode";

#[derive(Default)]
pub struct Base64DecodeNode;

impl Base64DecodeNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for Base64DecodeNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "Base64 解码".to_string(),
            en: "Base64 Decode".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHJlY3QgeD0iMyIgeT0iMyIgd2lkdGg9IjE4IiBoZWlnaHQ9IjE4IiByeD0iMiIvPjxwYXRoIGQ9Ik04IDEwaDgiLz48cGF0aCBkPSJNMTIgMTR2NCIvPjxwYXRoIGQ9Ik05LjUgMTUuNSAxMiAxOGwyLjUtMi41Ii8+PC9zdmc+",
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
            zh: "将 Base64 字符串解码为文本，包含 UTF-8 检查与长度信息。".to_string(),
            en: "Decode a Base64 string into text with UTF-8 validation and length info."
                .to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "decoded".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "解码得到的文本（非 UTF-8 会采用替换字符）".to_string(),
                    en: "Decoded text (non UTF-8 bytes are lossily converted).".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "is_utf8".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "解码结果是否是有效 UTF-8".to_string(),
                    en: "Whether the decoded bytes are valid UTF-8.".to_string(),
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
                    zh: "解码结果的字节长度".to_string(),
                    en: "Byte length of the decoded result.".to_string(),
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
                    zh: "需要解码的 Base64 字符串".to_string(),
                    en: "Base64 string to decode.".to_string(),
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
                    zh: "输入是否移除了末尾填充符号 \"=\"".to_string(),
                    en: "Input omits trailing padding characters (=).".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
        ]
    }
}
