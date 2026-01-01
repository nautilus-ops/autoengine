use rmcp::handler::server::tool::{DynCallToolHandler, ToolCallContext, ToolRoute};
use rmcp::model::{CallToolResult, Content, Icon, JsonObject, Tool, ToolAnnotations};
use rmcp::schemars::{JsonSchema, SchemaGenerator};
use rmcp::{ErrorData, RoleServer, Service};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use schemars::Schema;
use crate::mcp::state::McpState;

pub type ToolDefine<T> = ToolRoute<T>;

pub struct ToolCallBuilder {
    /// The name of the tool
    name: Cow<'static, str>,
    title: Option<String>,
    description: Option<Cow<'static, str>>,
    annotations: Option<ToolAnnotations>,
    icons: Option<Vec<Icon>>,
    call_func: Option<Arc<DynCallToolHandler<McpState>>>,
    input_schema: Option<JsonObject>,
    output_schema: Option<JsonObject>,
}

type ToolCallFunc<T, I> = dyn for<'s> Fn(
    ToolCallContext<'s, T>,
    I,
) -> Pin<
    Box<dyn Future<Output = Result<serde_json::Value, String>> + Send + Sync + 'static>,
> + Send
+ Sync;

impl ToolCallBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name: Cow::from(name),
            title: None,
            description: None,
            annotations: None,
            icons: None,
            call_func: None,
            input_schema: None,
            output_schema: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    pub fn with_icons(mut self, icons: Vec<Icon>) -> Self {
        self.icons = Some(icons);
        self
    }

    pub fn with_input<I: JsonSchema>(mut self) -> Self {
        let mut generator = SchemaGenerator::new(Default::default());
        let schema = I::json_schema(&mut generator);
        let input_schema = serde_json::to_value(schema)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        self.input_schema = Some(input_schema);
        self
    }

    pub fn with_input_schema(mut self, schema: Schema) -> Self {
        let input_schema = serde_json::to_value(schema)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        self.input_schema = Some(input_schema);
        self
    }
    pub fn with_output_schema<O: JsonSchema>(mut self) -> Self {
        let mut generator = SchemaGenerator::new(Default::default());
        let schema = O::json_schema(&mut generator);
        let output_schema = serde_json::to_value(schema)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        self.output_schema = Some(output_schema);
        self
    }
    pub fn with_call_func<I: JsonSchema + Serialize + DeserializeOwned + Default + 'static>(
        self,
        call_func: Arc<ToolCallFunc<McpState, I>>,
    ) -> Self {
        let mut builder = self;

        builder.call_func = Some(Arc::new(move |context| {
            let call_func = call_func.clone();
            Box::pin(async move {
                let obj = context.arguments.clone().unwrap_or_default();
                let arguments: I = serde_json::from_value(serde_json::Value::Object(obj))
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to convert argument {:?}", e);
                        Default::default()
                    });
                match call_func(context, arguments).await {
                    Ok(res) => Ok(CallToolResult::success(vec![
                        Content::json(res).unwrap_or(Content::text("Failed to call tool")),
                    ])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
                }
            })
        }));
        builder
    }

    pub fn build(self) -> ToolDefine<McpState> {
        let output_schema = if let Some(output_schema) = self.output_schema {
            Some(Arc::new(output_schema))
        } else {
            None
        };

        ToolDefine {
            call: self
                .call_func
                .unwrap_or(Arc::new(|_| panic!("call_func is not set"))),
            attr: Tool {
                name: self.name,
                title: self.title,
                description: self.description,
                input_schema: Arc::new(self.input_schema.unwrap_or(JsonObject::new())),
                output_schema,
                annotations: self.annotations,
                icons: self.icons,
                meta: None,
            },
        }
    }
}
