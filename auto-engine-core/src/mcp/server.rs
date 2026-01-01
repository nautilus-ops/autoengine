use crate::mcp::service::McpServiceBuilder;
use crate::mcp::state::McpState;
use crate::mcp::tool::{ToolCallBuilder, ToolDefine};
use crate::node::start::node;
use crate::schema::workflow::WorkflowSchema;
use crate::types::workflow::WorkflowMetaData;
use rmcp::handler::server::wrapper::Parameters;
use schemars::{Schema, SchemaGenerator};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

struct ParamSchemaBuilder {
    params: Vec<String>,
}

impl ParamSchemaBuilder {
    fn new() -> ParamSchemaBuilder {
        ParamSchemaBuilder { params: vec![] }
    }

    fn add_param(&mut self, name: String) {
        self.params.push(name);
    }

    fn build(self) -> Schema {
        let mut type_string = Map::new();
        type_string.insert("type".to_string(), Value::String("string".to_string()));
        let mut params = Map::new();
        let mut required = vec![];

        for param in self.params {
            params.insert(param.clone(), Value::Object(type_string.clone()));
            required.push(Value::String(param.clone()));
        }

        let mut res = Map::new();
        res.insert("properties".to_string(), Value::Object(params));
        res.insert("type".to_string(), Value::String("object".to_string()));
        res.insert("required".to_string(), Value::Array(required));

        Schema::from(res)
    }
}

pub struct McpServer {
    workflow_dir: PathBuf,
    token: CancellationToken,
}

impl McpServer {
    pub fn new(workflow_dir: PathBuf) -> Self {
        McpServer {
            workflow_dir,
            token: Default::default(),
        }
    }

    pub fn load_tools(&self) -> Result<Vec<ToolDefine<McpState>>, String> {
        if !self.workflow_dir.exists() {
            return Ok(vec![]);
        }

        let mut workflows = vec![];

        for entry in self.workflow_dir.read_dir().map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                // Try to load metadata from this directory
                let meta_path = path.join("_meta.toml");
                if !meta_path.exists() {
                    continue;
                }
                // read metadata
                let meta_content = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
                let meta = toml::from_str::<WorkflowMetaData>(meta_content.as_str())
                    .map_err(|e| e.to_string())?;

                // read workflow content
                let workflow_content =
                    fs::read_to_string(path.join("workflow.yaml")).map_err(|e| e.to_string())?;

                let schema = serde_yaml::from_str::<WorkflowSchema>(&workflow_content)
                    .map_err(|e| e.to_string())?;

                let mut builder = ParamSchemaBuilder::new();
                for node in schema.nodes {
                    if node.action_type == node::START_NODE_TYPE {
                        if let Some(value) = node.input_data.unwrap_or(HashMap::new()).get("params")
                        {
                            value.as_object().map(|obj| {
                                for (key, value) in obj {
                                    if value.is_string() {
                                        builder.add_param(key.clone())
                                    }
                                }
                            });
                        }
                    }
                }

                workflows.push((meta, builder.build()));
            }
        }

        let mut tools = vec![];

        for (workflow, param) in workflows {
            let tool = ToolCallBuilder::new(
                workflow
                    .name
                    .ok_or("No name found in metadata".to_string())?,
            )
            .with_description(
                workflow
                    .description
                    .ok_or("No description found in metadata".to_string())?,
            )
            .with_input_schema(param)
            .with_call_func(Arc::new(|ctx, params: HashMap<String, Value>| {
                Box::pin(async move { Ok(serde_json::Value::String("Hello !".to_string())) })
            }))
            .build();
            tools.push(tool);
        }

        Ok(tools)
    }

    pub async fn run(&self) -> Result<(), String> {
        let tools = self.load_tools()?;

        let service = McpServiceBuilder::new()
            .with_port(8080)
            .with_tool_calls(tools)
            .build();

        log::info!("MCP server listening on {}", self.workflow_dir.display());
        tracing::info!("MCP server started");
        service
            .run(self.token.clone())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn stop(&mut self) {
        self.token.cancel();
        self.token = Default::default();
    }

    pub async fn restart(&mut self) -> Result<(), String> {
        self.stop().await;
        self.run().await
    }
}
