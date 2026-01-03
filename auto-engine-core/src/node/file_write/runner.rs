use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileWriteParams {
    pub content: String,
    #[serde(default)]
    pub open_dir: bool,
    #[serde(default)]
    pub filename: String,
    pub filetype: String,
}

#[derive(Default)]
pub struct FileWriteRunner;

impl FileWriteRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn resolve_filename(&self, raw: &str, extension: &str) -> String {
        let trimmed = raw.trim();
        let base = if trimmed.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            Path::new(trimmed)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| Uuid::new_v4().to_string())
        };

        format!("{}.{}", base, extension)
    }

    fn normalize_extension(&self, ext: &str) -> Result<String, String> {
        let lower = ext.trim().to_lowercase();
        let allowed = ["md", "txt", "xml", "toml", "yaml", "html"];
        if allowed.contains(&lower.as_str()) {
            Ok(lower)
        } else {
            Err(format!(
                "Unsupported filetype '{}'. Allowed: md, txt, xml, toml, yaml, html",
                ext
            ))
        }
    }

    fn open_directory(path: &Path) {
        let dir = path.to_path_buf();
        #[cfg(target_os = "macos")]
        let cmd = Command::new("open").arg(dir.clone()).spawn();

        #[cfg(target_os = "windows")]
        let cmd = Command::new("explorer").arg(dir.clone()).spawn();

        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        let cmd = Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "open_dir not supported on this OS",
        ));

        if let Err(err) = cmd {
            warn!("Failed to open directory {:?}: {}", dir, err);
        }
    }
}

#[async_trait::async_trait]
impl NodeRunner for FileWriteRunner {
    type ParamType = FileWriteParams;

    async fn run(
        &mut self,
        ctx: &Context,
        params: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let ext = self.normalize_extension(&params.filetype)?;
        let filename = self.resolve_filename(&params.filename, &ext);

        let dir = ctx
            .path_files()
            .map_err(|e| format!("Failed to resolve files directory: {}", e))?;
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create files directory: {}", e))?;

        let filepath = dir.join(&filename);
        let content = params
            .content
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\\"", "\"");
        fs::write(&filepath, content).map_err(|e| format!("Failed to write file: {}", e))?;

        if params.open_dir {
            Self::open_directory(dir.as_path());
        }

        let mut result = HashMap::new();
        result.insert(
            "filepath".to_string(),
            serde_json::json!(filepath.to_string_lossy().to_string()),
        );
        result.insert("filename".to_string(), serde_json::json!(filename));

        Ok(Some(result))
    }
}

pub struct FileWriteRunnerFactory;

impl FileWriteRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FileWriteRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for FileWriteRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(FileWriteRunner::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_ctx() -> (Context, PathBuf) {
        let base = std::env::temp_dir().join(format!("autoengine-filewrite-{}", Uuid::new_v4()));
        fs::create_dir_all(&base.join("files")).unwrap();
        let ctx = Context::new(base.clone(), None);
        (ctx, base)
    }

    #[tokio::test]
    async fn writes_with_generated_name() {
        let (ctx, base) = temp_ctx();
        let mut runner = FileWriteRunner::new();
        let params = FileWriteParams {
            content: "hello".to_string(),
            open_dir: false,
            filename: "".to_string(),
            filetype: "txt".to_string(),
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        let filepath = result
            .get("filepath")
            .and_then(|v| v.as_str())
            .expect("filepath");
        let filename = result
            .get("filename")
            .and_then(|v| v.as_str())
            .expect("filename");

        assert!(filename.ends_with(".txt"));
        assert!(Path::new(filepath).exists());

        // cleanup
        let _ = fs::remove_dir_all(base);
    }

    #[tokio::test]
    async fn writes_with_custom_name_and_type() {
        let (ctx, base) = temp_ctx();
        let mut runner = FileWriteRunner::new();
        let params = FileWriteParams {
            content: "<root></root>".to_string(),
            open_dir: false,
            filename: "note.xml".to_string(),
            filetype: "xml".to_string(),
        };

        let result = runner.run(&ctx, params).await.unwrap().unwrap();
        let filepath = result
            .get("filepath")
            .and_then(|v| v.as_str())
            .expect("filepath");
        let filename = result
            .get("filename")
            .and_then(|v| v.as_str())
            .expect("filename");

        assert_eq!(filename, "note.xml");
        assert!(Path::new(filepath).exists());

        // cleanup
        let _ = fs::remove_dir_all(base);
    }
}
