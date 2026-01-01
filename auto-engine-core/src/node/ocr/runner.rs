use crate::context::Context;
use crate::types::node::{NodeRunner, NodeRunnerControl, NodeRunnerController, NodeRunnerFactory};
use oar_ocr::prelude::{OAROCRBuilder, load_image};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OcrParams {
    pub image: String,
    pub digits_only: bool,
}

#[derive(Default, Clone)]
pub struct OcrRunner;

impl OcrRunner {
    pub fn new() -> Self {
        Self {}
    }

    fn resolve_path(base: &Path, image: &str) -> PathBuf {
        let path = PathBuf::from(image);
        if path.is_absolute() {
            path
        } else {
            base.join(path)
        }
    }

    fn extract_digits(&self, s: &str) -> String {
        s.chars().filter(|c| c.is_ascii_digit()).collect()
    }
}

#[async_trait::async_trait]
impl NodeRunner for OcrRunner {
    type ParamType = OcrParams;

    async fn run(
        &mut self,
        ctx: &Context,
        param: Self::ParamType,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        let mut resource_path_prefix = ctx.path_resource().to_string_lossy().replace(r"\\?\", "").to_string();

        if resource_path_prefix != "" {
            resource_path_prefix = format!("{}/", resource_path_prefix);
        }

        let ocr = OAROCRBuilder::new(
            format!("{}{}", resource_path_prefix, "ocr/pp-ocrv5_mobile_det.onnx").as_str(),
            format!("{}{}", resource_path_prefix, "ocr/pp-ocrv5_mobile_rec.onnx").as_str(),
            format!("{}{}", resource_path_prefix, "ocr/ppocrv5_dict.txt").as_str(),
        )
        .build()
        .map_err(|e| {
            e.to_string()
        })?;

        let image_path = Self::resolve_path(&ctx.workflow_path.join("files"), &param.image);

        let image = load_image(&image_path).map_err(|e| e.to_string())?;
        let results = ocr.predict(vec![image]).map_err(|e| e.to_string())?;
        let mut res = HashMap::new();

        let mut text_res = "".to_string();
        for ocr_res in &results {
            for text_region in ocr_res.text_regions.clone() {
                if let Some((text, _)) = text_region.text_with_confidence() {
                    let text = if param.digits_only {
                        self.extract_digits(text)
                    } else {
                        text.to_string()
                    };
                    text_res.push_str(&text);
                    log::info!("======> {}", text);
                    log::info!("======> {}", text_res);
                }
            }
        }
        res.insert("text".to_string(), serde_json::json!(text_res));

        Ok(Some(res))
    }
}

pub struct OcrRunnerFactory;

impl OcrRunnerFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for OcrRunnerFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRunnerFactory for OcrRunnerFactory {
    fn create(&self) -> Box<dyn NodeRunnerControl> {
        Box::new(NodeRunnerController::new(OcrRunner::new()))
    }
}
