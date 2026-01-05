use crate::node::ai::gpt::node::GptNode;
use crate::node::ai::gpt::runner::GptRunnerFactory;
use crate::node::base64_decode::node::Base64DecodeNode;
use crate::node::base64_decode::runner::Base64DecodeRunnerFactory;
use crate::node::base64_encode::node::Base64EncodeNode;
use crate::node::base64_encode::runner::Base64EncodeRunnerFactory;
use crate::node::file_write::node::FileWriteNode;
use crate::node::file_write::runner::FileWriteRunnerFactory;
use crate::node::data_aggregator::node::DataAggregatorNode;
use crate::node::data_aggregator::runner::DataAggregatorRunnerFactory;
use crate::node::http::node::HttpNode;
use crate::node::http::runner::HttpRunnerFactory;
use crate::node::image_match::node::ImageMatchNode;
use crate::node::image_match::runner::ImageMatchRunnerFactory;
use crate::node::keyboard::node::KeyboardNode;
use crate::node::keyboard::runner::KeyboardNodeFactory;
use crate::node::mouse_click::node::MouseClickNode;
use crate::node::mouse_click::runner::MouseClickNodeFactory;
use crate::node::mouse_move::node::MouseMoveNode;
use crate::node::mouse_move::runner::MouseMoveNodeFactory;
use crate::node::ocr::node::OcrNode;
use crate::node::ocr::runner::OcrRunnerFactory;
use crate::node::screen_capture::node::ScreenCaptureNode;
use crate::node::screen_capture::runner::ScreenCaptureRunnerFactory;
use crate::node::text_replace::node::TextReplaceNode;
use crate::node::text_replace::runner::TextReplaceRunnerFactory;
use crate::node::text_case::node::TextCaseConvertNode;
use crate::node::text_case::runner::TextCaseConvertRunnerFactory;
use crate::node::start::node::StartNode;
use crate::node::start::runner::StartRunnerFactory;
use crate::node::time_wait::node::TimeWaitNode;
use crate::node::time_wait::runner::TimeWaitRunnerFactory;
use crate::types::node::{NodeDefine, NodeRunnerControl, NodeRunnerFactory};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct NodeRegisterBus {
    nodes: HashMap<String, Arc<Box<dyn NodeDefine + Send + Sync>>>,
    runner_factories: HashMap<String, Arc<Box<dyn NodeRunnerFactory + Send + Sync>>>,
}

impl NodeRegisterBus {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            runner_factories: HashMap::new(),
        }
    }

    pub fn with_internal_nodes(mut self) -> NodeRegisterBus {
        self.register(
            Box::new(StartNode::new()),
            Box::new(StartRunnerFactory::new()),
        );
        self.register(
            Box::new(MouseMoveNode::new()),
            Box::new(MouseMoveNodeFactory::new()),
        );
        self.register(
            Box::new(MouseClickNode::new()),
            Box::new(MouseClickNodeFactory::new()),
        );
        self.register(
            Box::new(KeyboardNode::new()),
            Box::new(KeyboardNodeFactory::new()),
        );
        self.register(
            Box::new(ImageMatchNode::new()),
            Box::new(ImageMatchRunnerFactory::new()),
        );
        self.register(
            Box::new(HttpNode::new()),
            Box::new(HttpRunnerFactory::new()),
        );
        self.register(
            Box::new(ScreenCaptureNode::new()),
            Box::new(ScreenCaptureRunnerFactory::new()),
        );
        self.register(Box::new(OcrNode::new()), Box::new(OcrRunnerFactory::new()));
        self.register(
            Box::new(TimeWaitNode::new()),
            Box::new(TimeWaitRunnerFactory::new()),
        );
        self.register(
            Box::new(DataAggregatorNode::new()),
            Box::new(DataAggregatorRunnerFactory::new()),
        );
        self.register(
            Box::new(Base64EncodeNode::new()),
            Box::new(Base64EncodeRunnerFactory::new()),
        );
        self.register(
            Box::new(Base64DecodeNode::new()),
            Box::new(Base64DecodeRunnerFactory::new()),
        );
        self.register(
            Box::new(FileWriteNode::new()),
            Box::new(FileWriteRunnerFactory::new()),
        );
        self.register(
            Box::new(TextCaseConvertNode::new()),
            Box::new(TextCaseConvertRunnerFactory::new()),
        );
        self.register(
            Box::new(TextReplaceNode::new()),
            Box::new(TextReplaceRunnerFactory::new()),
        );
        self.register(Box::new(GptNode::new()), Box::new(GptRunnerFactory::new()));
        self
    }

    pub fn register(&mut self, node: Box<dyn NodeDefine>, factory: Box<dyn NodeRunnerFactory>) {
        let key = node.action_type();

        self.nodes.insert(key.clone(), Arc::new(node));
        self.runner_factories.insert(key, Arc::new(factory));
    }

    pub fn register_runner(&mut self, action_type: String, factory: Box<dyn NodeRunnerFactory>) {
        let key = action_type.clone();
        self.runner_factories.insert(key, Arc::new(factory));
    }

    pub fn register_node(&mut self, action_type: String, node: Box<dyn NodeDefine>) {
        let key = action_type.clone();
        self.nodes.insert(key, Arc::new(node));
    }

    pub fn list_nodes(&self) -> Vec<Arc<Box<dyn NodeDefine + Send + Sync>>> {
        let mut res = vec![];
        for (_key, value) in self.nodes.iter() {
            res.push(Arc::clone(value));
        }
        res
    }

    pub fn load_node(&self, action_type: &str) -> Option<Arc<Box<dyn NodeDefine + Send + Sync>>> {
        Some(self.nodes.get(action_type)?.clone())
    }

    pub fn create_runner(&self, key: &str) -> Option<Box<dyn NodeRunnerControl>> {
        let factory = self.runner_factories.get(key)?.clone();
        Some(factory.create())
    }
}
