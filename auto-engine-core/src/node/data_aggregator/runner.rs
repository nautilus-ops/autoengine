use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataAggregatorParam {
    pub mode: String,
    pub sources: Vec<String>,
    #[serde(default)]
    pub keys: Vec<String>,
}

#[derive(Default)]
pub struct DataAggregatorRunner;

impl DataAggregatorRunner {
    pub fn new() -> Self {
        DataAggregatorRunner
    }
}

#[async_trait::async_trait]
impl NodeRunner for DataAggregatorRunner {
    type ParamType = DataAggregatorParam;

    async fn run(
        &mut self,
        ctx: &Context,
        param: Self::ParamType,
    ) -> Result<Option<HashMap<String, Value>>, String> {
        {
            let map = ctx.value.read().await;
            log::info!("map: {:?}", map.keys());
        }
        let mut values = vec![];

        for source in param.sources.iter() {
            match ctx.get_value_parse(source).await {
                Some(value) => values.push(value),
                None => {
                    values.push(Value::Null);
                }
            }
        }

        let count = values.len();
        let result = match param.mode.to_lowercase().as_str() {
            "array" => {
                // Array mode: return values as an array
                Value::Array(values)
            }
            "object" => {
                // Object mode: use keys if provided, otherwise use indices
                let mut map = serde_json::Map::new();
                for (i, value) in values.iter().enumerate() {
                    let key = if i < param.keys.len() {
                        param.keys[i].clone()
                    } else {
                        format!("item_{}", i)
                    };
                    map.insert(key, value.clone());
                }
                Value::Object(map)
            }
            _ => {
                return Err(format!("Invalid mode: {}", param.mode));
            }
        };

        let mut output = HashMap::new();
        output.insert("result".to_string(), result);
        output.insert("count".to_string(), Value::from(count));

        Ok(Some(output))
    }
}

#[derive(Default)]
pub struct DataAggregatorRunnerFactory;

impl DataAggregatorRunnerFactory {
    pub fn new() -> Self {
        DataAggregatorRunnerFactory
    }
}

impl NodeRunnerFactory for DataAggregatorRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(DataAggregatorRunner::new()))
    }
}
