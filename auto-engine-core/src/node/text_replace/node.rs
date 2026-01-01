use crate::types::field::{
    BooleanConstraint, Condition, FieldCondition, FieldType, SchemaField, ValueConstraint,
};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "TextReplace";

#[derive(Default)]
pub struct TextReplaceNode;

impl TextReplaceNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for TextReplaceNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "字符串替换".to_string(),
            en: "Text Replace".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgY2xhc3M9Imx1Y2lkZSBsdWNpZGUtdGV4dC1pbml0aWFsLWljb24gbHVjaWRlLXRleHQtaW5pdGlhbCI+PHBhdGggZD0iTTE1IDVoNiIvPjxwYXRoIGQ9Ik0xNSAxMmg2Ii8+PHBhdGggZD0iTTMgMTloMTgiLz48cGF0aCBkPSJtMyAxMiAzLjU1My03LjcyNGEuNS41IDAgMCAxIC44OTQgMEwxMSAxMiIvPjxwYXRoIGQ9Ik0zLjkyIDEwaDYuMTYiLz48L3N2Zz4=",
        )
    }

    fn category(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "数据处理".to_string(),
            en: "Data Processing".to_string(),
        })
    }

    fn description(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: "对文本内容执行字符串替换，支持普通模式或正则模式。".to_string(),
            en: "Perform string replacements on text with plain or regex mode.".to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "result".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "替换后的文本结果".to_string(),
                    en: "Text after replacement".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "replaced_count".to_string(),
                field_type: FieldType::Number,
                item_type: None,
                description: Some(I18nValue {
                    zh: "发生替换的次数".to_string(),
                    en: "Number of replacements performed".to_string(),
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
                name: "source".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "需要进行替换的原始文本内容".to_string(),
                    en: "Source text to process".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "pattern".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "要匹配的字符串或正则表达式".to_string(),
                    en: "Pattern string or regular expression".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "replacement".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "用于替换的文本内容".to_string(),
                    en: "Replacement text".to_string(),
                }),
                enums: vec![],
                default: Some("".to_string()),
                condition: None,
            },
            SchemaField {
                name: "use_regex".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "是否使用正则表达式匹配".to_string(),
                    en: "Use regular expression matching".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "case_insensitive".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "正则匹配是否忽略大小写".to_string(),
                    en: "Ignore case when using regex".to_string(),
                }),
                enums: vec![],
                default: Some("false".to_string()),
                condition: Some(Condition::Field(FieldCondition {
                    field: "use_regex".to_string(),
                    constraint: ValueConstraint::Boolean(BooleanConstraint { equals: true }),
                    required: false,
                })),
            },
            SchemaField {
                name: "replace_all".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "是否替换所有匹配项（否则仅替换第一个）".to_string(),
                    en: "Replace all matches (otherwise only the first one)".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
        ]
    }
}
