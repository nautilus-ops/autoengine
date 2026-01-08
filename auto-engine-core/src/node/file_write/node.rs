use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const NODE_TYPE: &str = "FileWrite";

#[derive(Default)]
pub struct FileWriteNode;

impl FileWriteNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeDefine for FileWriteNode {
    fn action_type(&self) -> String {
        NODE_TYPE.to_string()
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: "写入文件".to_string(),
            en: "Write File".to_string(),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgY2xhc3M9Imx1Y2lkZSBsdWNpZGUtbm90ZWJvb2stcGVuLWljb24gbHVjaWRlLW5vdGVib29rLXBlbiI+PHBhdGggZD0iTTEzLjQgMkg2YTIgMiAwIDAgMC0yIDJ2MTZhMiAyIDAgMCAwIDIgMmgxMmEyIDIgMCAwIDAgMi0ydi03LjQiLz48cGF0aCBkPSJNMiA2aDQiLz48cGF0aCBkPSJNMiAxMGg0Ii8+PHBhdGggZD0iTTIgMTRoNCIvPjxwYXRoIGQ9Ik0yIDE4aDQiLz48cGF0aCBkPSJNMjEuMzc4IDUuNjI2YTEgMSAwIDEgMC0zLjAwNC0zLjAwNGwtNS4wMSA1LjAxMmEyIDIgMCAwIDAtLjUwNi44NTRsLS44MzcgMi44N2EuNS41IDAgMCAwIC42Mi42MmwyLjg3LS44MzdhMiAyIDAgMCAwIC44NTQtLjUwNnoiLz48L3N2Zz4=",
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
            zh: "将文本写入文件，可自动生成文件名并可选择写入后打开目录。".to_string(),
            en: "Write text content to a file, auto-generate filename if empty, and optionally open the directory."
                .to_string(),
        })
    }

    fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        vec![
            SchemaField {
                name: "filepath".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "保存后的文件完整路径".to_string(),
                    en: "Full path of the saved file".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "filename".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "保存后的文件名".to_string(),
                    en: "Saved filename".to_string(),
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
                name: "content".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "要写入文件的文本内容".to_string(),
                    en: "Text content to write into the file".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "open_dir".to_string(),
                field_type: FieldType::Boolean,
                item_type: None,
                description: Some(I18nValue {
                    zh: "写入完成后是否打开目录".to_string(),
                    en: "Open the directory after writing".to_string(),
                }),
                enums: vec![],
                default: None,
                condition: None,
            },
            SchemaField {
                name: "filename".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "文件名（可空，为空将自动生成）".to_string(),
                    en: "Filename (optional; auto-generated when empty)".to_string(),
                }),
                enums: vec![],
                default: Some("".to_string()),
                condition: None,
            },
            SchemaField {
                name: "filetype".to_string(),
                field_type: FieldType::String,
                item_type: None,
                description: Some(I18nValue {
                    zh: "文件类型：md、txt、xml、toml、yaml、html".to_string(),
                    en: "File type: md, txt, xml, toml, yaml, html".to_string(),
                }),
                enums: vec![
                    "md".to_string(),
                    "txt".to_string(),
                    "xml".to_string(),
                    "toml".to_string(),
                    "yaml".to_string(),
                    "html".to_string(),
                ],
                default: Some("txt".to_string()),
                condition: None,
            },
        ]
    }
}
