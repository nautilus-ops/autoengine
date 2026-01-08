use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "TimeNow";

#[derive(Default)]
pub struct TimeNowNode;

impl TimeNowNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for TimeNowNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "获取当前时间".to_string(),
            en: "Current Time".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgY2xhc3M9Imx1Y2lkZSBsdWNpZGUtY2xvY2s0LWljb24gbHVjaWRlLWNsb2NrLTQiPjxwYXRoIGQ9Ik0xMiA2djZsNCAyIi8+PGNpcmNsZSBjeD0iMTIiIGN5PSIxMiIgcj0iMTAiLz48L3N2Zz4=",
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
            zh: "按指定时区与格式输出当前时间，并提供时间戳。".to_string(),
            en: "Outputs the current time using a chosen time zone and format, plus a timestamp."
                .to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "now".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "格式化后的当前时间".to_string(),
                    en: "Formatted current time.".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "timestamp".to_string(),
                field_type: FieldType::Number,
                item_type: None,
                description: Some(I18nValue {
                    zh: "当前时间的 Unix 时间戳（秒）".to_string(),
                    en: "Unix timestamp of the current time (seconds).".to_string(),
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
                name: "time_zone".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "时区，例如 UTC、Asia/Shanghai 或 +08:00".to_string(),
                    en: "Time zone, e.g. UTC, Asia/Shanghai, or +08:00.".to_string(),
                }),
                enums: vec![],
                default: Some("UTC".to_string()),
                condition: None,
            },
            SchemaField {
                name: "format".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "时间格式字符串（基于 strftime 规则）".to_string(),
                    en: "Time format string (strftime-compatible).".to_string(),
                }),
                enums: vec![],
                default: Some("%Y-%m-%d %H:%M:%S".to_string()),
                condition: None,
            },
        ]
    }
}
