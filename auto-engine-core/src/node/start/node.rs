use crate::types::field::{FieldType, SchemaField};
use crate::types::node::{I18nValue, NodeDefine};
use std::collections::HashMap;

pub const START_NODE_TYPE: &'static str = "Start";

pub struct StartNode;

impl StartNode {
    pub(crate) fn new() -> StartNode {
        Self {}
    }
}

impl NodeDefine for StartNode {
    fn action_type(&self) -> String {
        String::from(START_NODE_TYPE)
    }

    fn name(&self) -> I18nValue {
        I18nValue {
            zh: String::from("开始"),
            en: String::from("Start"),
        }
    }

    fn icon(&self) -> String {
        String::from(
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgY2xhc3M9Imx1Y2lkZSBsdWNpZGUtY2lyY2xlLXBsYXktaWNvbiBsdWNpZGUtY2lyY2xlLXBsYXkiPjxwYXRoIGQ9Ik05IDkuMDAzYTEgMSAwIDAgMSAxLjUxNy0uODU5bDQuOTk3IDIuOTk3YTEgMSAwIDAgMSAwIDEuNzE4bC00Ljk5NyAyLjk5N0ExIDEgMCAwIDEgOSAxNC45OTZ6Ii8+PGNpcmNsZSBjeD0iMTIiIGN5PSIxMiIgcj0iMTAiLz48L3N2Zz4=",
        )
    }

    fn category(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: String::from("基础节点"),
            en: String::from("Basic Node"),
        })
    }

    fn description(&self) -> Option<I18nValue> {
        Some(I18nValue {
            zh: String::from("工作流从此节点开始执行"),
            en: String::from("The workflow start at this node."),
        })
    }

    fn output_schema(&self, input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
        let val = input.get("params").unwrap_or_default().clone();
        let params: HashMap<String, serde_json::Value> =
            serde_json::from_value(val).unwrap_or_default();

        let mut outputs = vec![];
        for (key, _value) in params.iter() {
            outputs.push(SchemaField {
                name: key.to_string(),
                field_type: Default::default(),
                item_type: None,
                description: None,
                enums: vec![],
                default: None,
                condition: None,
            });
        }

        outputs
    }

    fn input_schema(&self) -> Vec<SchemaField> {
        vec![SchemaField {
            name: "params".to_string(),
            field_type: FieldType::Object,
            item_type: None,
            description: Some(I18nValue {
                zh: "".to_string(),
                en: "".to_string(),
            }),
            enums: vec![],
            default: None,
            condition: None,
        }]
    }
}
