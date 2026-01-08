use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use async_openai::Client;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GptParams {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "GptParams::default_model")]
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub system_prompt_enabled: bool,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub response_format_enabled: bool,
    pub response_json_schema: Option<String>,
}

impl GptParams {
    fn default_model() -> String {
        "gpt-4o-mini".to_string()
    }
}

#[derive(Default, Clone)]
pub struct GptRunner;

impl GptRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn resolve_api_key(&self, provided: Option<String>) -> Option<String> {
        match provided {
            Some(key) if !key.trim().is_empty() => Some(key.trim().to_string()),
            _ => std::env::var("OPENAI_API_KEY").ok(),
        }
    }

    async fn build_context_prompt(&self, ctx: &Context) -> String {
        format!(
            "You are an AI node in the workflow. You can view the output from the preceding node {:?}, formatted as `ctx.{{node_name}}.{{node_output_key}}`.  Simply retrieve the value corresponding to the key. Unless otherwise specified, please output using the following response JSON format: `{{\"data\": value}}`.",
            ctx.values().await
        )
    }

    pub fn default_response_json_schema(&self) -> ResponseFormatJsonSchema {
        ResponseFormatJsonSchema {
            description: Some(String::from(
                "The response data can be of any type, including primitive types, arrays, objects, etc.",
            )),
            name: "data".to_string(),
            schema: Some(json!({
                "type": "object",
                "properties": {}
            })),
            strict: None,
        }
    }
}

#[async_trait::async_trait]
impl NodeRunner for GptRunner {
    type ParamType = GptParams;

    async fn run(
        &mut self,
        ctx: &Context,
        param: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        if param.prompt.trim().is_empty() {
            return Err("prompt cannot be empty".to_string());
        }

        let api_key = self.resolve_api_key(param.api_key).ok_or_else(|| {
            "OpenAI API key is missing; provide api_key or set OPENAI_API_KEY".to_string()
        })?;

        let mut config = OpenAIConfig::default().with_api_key(api_key);
        if let Some(base_url) = param
            .base_url
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            config = config.with_api_base(base_url);
        }

        if config.api_key().expose_secret().is_empty() {
            return Err("OpenAI API key is empty; please configure it first".to_string());
        }

        let client = Client::with_config(config);

        let mut messages: Vec<ChatCompletionRequestMessage> = vec![];

        // context prompt
        // {
        //     let context_prompt = self.build_context_prompt(ctx).await;
        //     messages.push(
        //         ChatCompletionRequestSystemMessageArgs::default()
        //             .content(context_prompt)
        //             .build()
        //             .map_err(|e| format!("failed to build system message: {}", e))?
        //             .into(),
        //     );
        // }

        {
                messages.push(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(String::from("The generated content needs to be placed in the data section: {\"data\": object}"))
                        .build()
                        .map_err(|e| format!("failed to build system message: {}", e))?
                        .into(),
                );
        }
        // user set system prompt
        if let Some(system_prompt) = param
            .system
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            && param.system_prompt_enabled
        {
            let system_message = ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt.to_string())
                .build()
                .map_err(|e| format!("failed to build system message: {}", e))?;
            messages.push(ChatCompletionRequestSystemMessage::from(system_message).into());
        }

        let user_message = ChatCompletionRequestUserMessageArgs::default()
            .content(param.prompt)
            .build()
            .map_err(|e| format!("failed to build user message: {}", e))?;
        messages.push(ChatCompletionRequestUserMessage::from(user_message).into());

        let mut response_schema = self.default_response_json_schema();
        if let Some(json_schema) = param.response_json_schema.clone()
            && param.response_format_enabled
        {
            response_schema.schema =
                Some(serde_json::from_str(&json_schema).map_err(|e| {
                    format!("failed to parse response JSON schema: {}", e.to_string())
                })?);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(param.model)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: response_schema,
            })
            .messages(messages)
            .build()
            .map_err(|e| format!("failed to build chat request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI chat request failed: {}", e))?;

        let choice = response
            .choices
            .first()
            .ok_or_else(|| "OpenAI chat request failed: no choices returned".to_string())?;

        let content = choice.message.content.clone().ok_or_else(|| {
            log::error!("OpenAI chat request failed: no message content was returned");
            "OpenAI chat request failed: no message content was returned".to_string()
        })?;

        let mut res = HashMap::new();

        if content.trim().is_empty() {
            res.insert("content".to_string(), serde_json::json!(content));
        } else {
            let data: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(content.as_str())
                    .inspect_err(|e| log::error!("chat message: {}, error: {}", content, e))
                    .map_err(|e| format!("failed to parse chat message: {}", e))?;

            log::info!("OpenAI chat message: {}", content);
            res.insert("data".to_string(), serde_json::json!(data.get("data")));
        }

        let usage = response.usage.unwrap_or_default();
        res.insert(
            "prompt_tokens".to_string(),
            serde_json::json!(usage.prompt_tokens),
        );
        res.insert(
            "completion_tokens".to_string(),
            serde_json::json!(usage.completion_tokens),
        );
        res.insert(
            "total_tokens".to_string(),
            serde_json::json!(usage.total_tokens),
        );

        Ok(Some(res))
    }
}

pub struct GptRunnerFactory;

impl GptRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GptRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for GptRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(GptRunner::new()))
    }
}
