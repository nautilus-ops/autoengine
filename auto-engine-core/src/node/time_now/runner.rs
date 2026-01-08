use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use chrono::{FixedOffset, Local, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

fn default_time_zone() -> String {
    "UTC".to_string()
}

fn default_format() -> String {
    "%Y-%m-%d %H:%M:%S".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeNowParam {
    #[serde(default = "default_time_zone")]
    pub time_zone: String,
    #[serde(default = "default_format")]
    pub format: String,
}

enum ResolvedTimeZone {
    Named(Tz),
    Offset(FixedOffset),
    Local,
}

#[derive(Default)]
pub struct TimeNowRunner;

impl TimeNowRunner {
    pub fn new() -> Self {
        TimeNowRunner
    }

    fn resolve_time_zone(&self, input: &str) -> Result<ResolvedTimeZone, String> {
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("local") {
            return Ok(ResolvedTimeZone::Local);
        }
        if let Ok(tz) = trimmed.parse::<Tz>() {
            return Ok(ResolvedTimeZone::Named(tz));
        }
        if let Ok(offset) = FixedOffset::from_str(trimmed) {
            return Ok(ResolvedTimeZone::Offset(offset));
        }

        Err(format!(
            "Unsupported time zone: {}. Try values like UTC, Asia/Shanghai, local, or +08:00",
            trimmed
        ))
    }
}

#[async_trait::async_trait]
impl NodeRunner for TimeNowRunner {
    type ParamType = TimeNowParam;

    async fn run(
        &mut self,
        _ctx: &Context,
        param: Self::ParamType,
    ) -> Result<Option<HashMap<String, Value>>, String> {
        let resolved = self.resolve_time_zone(&param.time_zone)?;
        let now = Utc::now();

        let (formatted, timestamp) = match resolved {
            ResolvedTimeZone::Named(tz) => {
                let dt = now.with_timezone(&tz);
                (dt.format(&param.format).to_string(), dt.timestamp())
            }
            ResolvedTimeZone::Offset(offset) => {
                let dt = now.with_timezone(&offset);
                (dt.format(&param.format).to_string(), dt.timestamp())
            }
            ResolvedTimeZone::Local => {
                let dt = Local::now();
                (dt.format(&param.format).to_string(), dt.timestamp())
            }
        };

        let mut output = HashMap::new();
        output.insert("now".to_string(), Value::String(formatted));
        output.insert("timestamp".to_string(), Value::from(timestamp));

        Ok(Some(output))
    }
}

#[derive(Default)]
pub struct TimeNowRunnerFactory;

impl TimeNowRunnerFactory {
    pub fn new() -> Self {
        TimeNowRunnerFactory
    }
}

impl NodeRunnerFactory for TimeNowRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(TimeNowRunner::new()))
    }
}
