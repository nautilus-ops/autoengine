use crate::mcp::state::McpState;
use crate::mcp::tool::ToolDefine;
use axum::Router;
use rmcp::handler::server::tool::{ToolCallContext, ToolRoute};
use rmcp::transport::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::{RoleServer, Service};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct McpServiceBuilder {
    // service: Option<McpState>,
    port: Option<u16>,
    host: Option<String>,
    log_level: Option<String>,
    tool_calls: Vec<ToolDefine<McpState>>,
}

impl McpServiceBuilder {
    pub fn new() -> Self {
        McpServiceBuilder {
            port: None,
            host: None,
            log_level: None,
            tool_calls: vec![],
        }
    }
    pub fn with_port(mut self, port: u16) -> Self {
        log::info!("port ===> {}", port);
        self.port = Some(port);
        self
    }

    pub fn with_host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn with_log_level(mut self, log_level: &str) -> Self {
        self.log_level = Some(log_level.to_string());
        self
    }

    pub fn with_tool_call(mut self, tool_call: ToolDefine<McpState>) -> Self {
        self.tool_calls.push(tool_call);
        self
    }
    pub fn with_tool_calls(mut self, tool_call: Vec<ToolDefine<McpState>>) -> Self {
        log::info!("tool_call ===> {}", tool_call.len());
        self.tool_calls.extend(tool_call);
        self
    }

    pub fn build(&self) -> McpService<McpState> {
        log::info!("build ===>");

        let log_level = self.log_level.clone().unwrap_or("debug".to_string());

        // tracing_subscriber::registry()
        //     .with(
        //         tracing_subscriber::EnvFilter::try_from_default_env()
        //             .unwrap_or_else(|_| log_level.into()),
        //     )
        //     .with(tracing_subscriber::fmt::layer())
        //     .init();

        let tool_calls = self.tool_calls.clone();

        let service = StreamableHttpService::new(
            move || {
                let mut state = McpState::new();

                for tool in tool_calls.clone() {
                    state
                        .tool_router
                        .map
                        .insert(tool.attr.name.clone(), tool.clone());
                }

                Ok(state)
            },
            LocalSessionManager::default().into(),
            Default::default(),
        );

        McpService::Streamable(service, self.port.clone())
    }
}

pub enum McpService<S: Service<RoleServer>> {
    Streamable(StreamableHttpService<S>, Option<u16>),
}

impl McpService<McpState> {
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            McpService::Streamable(service, port) => {
                let mut p = 23456;
                if let Some(port) = port {
                    p = port
                }
                let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", p))?;
                let tcp_listener = tokio::net::TcpListener::bind(addr).await?;
                tracing::info!("listening on {}", addr);
                let router = Router::new().nest_service("/mcp", service);
                let _ = axum::serve(tcp_listener, router)
                    .with_graceful_shutdown(async move {
                        select! {
                            _ = cancel.cancelled() => {}
                            _ = tokio::signal::ctrl_c() => {}
                        }
                    })
                    .await;
                Ok(())
            }
        }
    }
}
