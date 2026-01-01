use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;
use tauri::async_runtime::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueItem {
    pub description: String,
    pub value: serde_json::Value,
}

#[derive(Debug)]
pub struct Context {
    pub value: Arc<RwLock<HashMap<String, ValueItem>>>,
    pub(crate) screen_scale: f64,
    pub(crate) pipeline_path: PathBuf,
    pub(crate) workflow_path: PathBuf,
    #[cfg(feature = "tauri")]
    pub(crate) app_handle: Option<tauri::AppHandle>,
}

impl Context {
    #[cfg(feature = "tauri")]
    pub fn new(path: PathBuf, app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            value: Arc::new(RwLock::new(HashMap::new())),
            screen_scale: 1.0,
            pipeline_path: path.clone(),
            workflow_path: path.clone(),
            app_handle,
        }
    }

    #[cfg(not(feature = "tauri"))]
    pub fn new(path: PathBuf) -> Self {
        Self {
            value: Arc::new(RwLock::new(HashMap::new())),
            screen_scale: 1.0,
            pipeline_path: path.clone(),
            workflow_path: path.clone(),
        }
    }

    pub fn with_screen_scale(mut self, screen_scale: f64) -> Self {
        self.screen_scale = screen_scale;
        self
    }

    pub async fn set_string_value(&self, key: &str, value: &str) -> Result<(), String> {
        self.set_value::<String>(key, value.to_string(), String::new())
            .await
    }

    pub async fn set_value<T: Serialize>(
        &self,
        key: &str,
        value: T,
        description: String,
    ) -> Result<(), String> {
        let mut map = self.value.write().await;
        map.insert(
            key.to_string(),
            ValueItem {
                description,
                value: serde_json::to_value(value).map_err(|e| format!("{:?}", e))?,
            },
        );
        Ok(())
    }
    pub async fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        let map = self.value.read().await;
        if let Some(item) = map.get(key).cloned() {
            return Some(item.value);
        }
        None
    }

    pub async fn get_value_parse(&self, key: &str) -> Option<serde_json::Value> {
        let mut default_value = None;
        let mut key = key.to_string();
        if let Some(caps) = utils::REGEX_PARSE_VARIABLES.captures_iter(&key).next() {
            let var_name = (caps[1]).to_string();
            let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            if !default.is_empty() {
                default_value = Some(serde_json::Value::String(default.to_string()));
            }
            key = var_name;
        }

        let value = self.get_value(&key).await;
        if value.is_some() {
            return value;
        }

        default_value
    }

    pub fn path_image(&self, image: &str) -> Result<PathBuf, String> {
        let image_path = self.workflow_path.join("images").join(image);
        if !image_path.exists() {
            return Err(format!("Image {} does not exist", image));
        }
        Ok(image_path)
    }

    pub fn path_resource(&self) -> PathBuf {
        if let Some(handle) = self.app_handle.clone() {
            if cfg!(debug_assertions) {
                return PathBuf::from("");
            }
            return handle.path().resource_dir().unwrap().to_path_buf();
        }
        PathBuf::from("")
    }

    pub async fn values(&self) -> HashMap<String, ValueItem> {
        let mut res: HashMap<String, ValueItem> = HashMap::new();
        let map = self.value.read().await;
        map.iter().for_each(|(k, v)| {
            res.insert(k.clone(), v.clone());
        });
        res
    }
}
