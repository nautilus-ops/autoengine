use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until};
use tokio_util::sync::CancellationToken;

use crate::workflow::BoxFuture;
use crate::{
    context::Context,
    event::{NODE_EVENT, NodeEventPayload, WORKFLOW_EVENT, WorkflowEventPayload, WorkflowStatus},
    notification::emitter::NotificationEmitter,
    register::bus::NodeRegisterBus,
    schema::{node::NodeSchema, workflow::WorkflowSchema},
};

#[derive(Debug, Clone)]
struct GraphNode {
    pub node_id: String,
    pub node_context: NodeSchema,
    pub next: Vec<Arc<std::sync::RwLock<GraphNode>>>,
    pub wait_count: Arc<AtomicUsize>,
}

fn build_graph(
    node_id: &str,
    graph_nodes: &HashMap<String, Arc<std::sync::RwLock<GraphNode>>>,
    edges: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
) -> Result<(), String> {
    if visited.contains(node_id) {
        return Ok(());
    }

    if !visiting.insert(node_id.to_string()) {
        return Err(format!("workflow contains a cycle at node '{node_id}'"));
    }

    let Some(next_edges) = edges.get(node_id) else {
        visited.insert(node_id.to_string());
        visiting.remove(node_id);
        return Ok(());
    };

    let mut next_nodes = vec![];
    for next_node_id in next_edges.iter() {
        let rc_node = graph_nodes
            .get(next_node_id)
            .ok_or_else(|| format!("connection references missing node '{next_node_id}'"))?
            .clone();

        {
            let node_writer = rc_node.write().map_err(|e| e.to_string())?;
            node_writer
                .wait_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        build_graph(next_node_id, graph_nodes, edges, visited, visiting)?;

        next_nodes.push(rc_node);
    }

    if let Some(node) = graph_nodes.get(node_id) {
        let mut node = node.write().unwrap();
        node.next = next_nodes;
    }

    visiting.remove(node_id);
    visited.insert(node_id.to_string());
    Ok(())
}

#[derive(Debug)]
pub struct WorkflowRunner {
    graph: Vec<Arc<std::sync::RwLock<GraphNode>>>,
}

impl WorkflowRunner {
    pub fn create(workflow: WorkflowSchema) -> Result<Self, String> {
        let mut graph_nodes: HashMap<String, Arc<std::sync::RwLock<GraphNode>>> = HashMap::new();
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        let mut start_nodes = vec![];

        for node_context in workflow.nodes.into_iter() {
            let key = node_context.node_id.clone();
            if node_context.action_type == "Start" {
                start_nodes.push(key.clone());
            }

            graph_nodes.insert(
                key.clone(),
                Arc::new(std::sync::RwLock::new(GraphNode {
                    node_id: key,
                    node_context,
                    next: vec![],
                    wait_count: Arc::new(AtomicUsize::new(0)),
                })),
            );
        }

        for edge in workflow.connections.into_iter() {
            for start in start_nodes.iter() {
                if start == &edge.to {
                    return Err(
                        "The [Start] node cannot be used as the next node in the connection."
                            .to_string(),
                    );
                }
            }

            if !graph_nodes.contains_key(&edge.from) {
                return Err(format!(
                    "connection references missing node '{}'",
                    edge.from
                ));
            }
            if !graph_nodes.contains_key(&edge.to) {
                return Err(format!("connection references missing node '{}'", edge.to));
            }

            let entry = edges.entry(edge.from).or_default();
            entry.push(edge.to);
        }

        if start_nodes.is_empty() {
            return Err("workflow missing Start node".to_string());
        }

        let mut graph: Vec<Arc<std::sync::RwLock<GraphNode>>> = vec![];
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        for key in start_nodes.iter() {
            build_graph(key, &graph_nodes, &edges, &mut visited, &mut visiting)?;
            if let Some(node) = graph_nodes.get(key) {
                graph.push(node.clone());
            }
        }

        Ok(Self { graph })
    }

    pub async fn run(
        &self,
        ctx: Arc<Context>,
        token: CancellationToken,
        bus: Arc<RwLock<NodeRegisterBus>>,
        emitter: Arc<NotificationEmitter>,
    ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
        emitter.clone().emit(
            WORKFLOW_EVENT,
            WorkflowEventPayload {
                status: WorkflowStatus::Running,
            },
        )?;

        let result = handle_nod(self.graph.clone(), ctx, token, bus, emitter.clone()).await;

        log::info!("workflow finished");
        emitter
            .emit(
                WORKFLOW_EVENT,
                WorkflowEventPayload {
                    status: WorkflowStatus::Finished,
                },
            )
            .unwrap_or_default();
        emitter
            .emit(NODE_EVENT, NodeEventPayload::cancel())
            .unwrap_or_default();

        log::info!("workflow finished, the result: {:?}", result);

        result
    }
}

type WorkflowResult = Result<Option<HashMap<String, serde_json::Value>>, String>;

fn handle_nod(
    graph: Vec<Arc<std::sync::RwLock<GraphNode>>>,
    ctx: Arc<Context>,
    token: CancellationToken,
    bus: Arc<RwLock<NodeRegisterBus>>,
    emitter: Arc<NotificationEmitter>,
) -> BoxFuture<WorkflowResult> {
    Box::pin(async move {
        let mut tasks: JoinSet<Result<Option<HashMap<String, serde_json::Value>>, String>> =
            JoinSet::new();
        for node in graph.iter() {
            let token_clone = token.clone();
            let token = token.clone();
            let bus = bus.clone();
            let ctx = ctx.clone();
            let emitter_clone = emitter.clone();
            let emitter = emitter.clone();

            let (node_id, node_schema, next_node, wait_count) = {
                let node_read = node.read().map_err(|e| e.to_string())?;
                (
                    node_read.node_id.clone(),
                    node_read.node_context.clone(),
                    node_read.next.clone(),
                    node_read.wait_count.clone(),
                )
            };

            let action = node_schema.action_type.clone();

            let (node, mut runner) = {
                let locked_bus = bus.read().await;
                let node = match locked_bus.load_node(&action) {
                    None => {
                        return Err(format!("Can't find node : {}", action));
                    }
                    Some(node) => node,
                };
                let runner = match locked_bus.create_runner(&action) {
                    Some(runner) => runner,
                    None => {
                        return Err(format!("Can't find action runner for node: {}", action));
                    }
                };
                (node, runner)
            };
            let run_input = node_schema.input_data.clone().unwrap_or_default();
            let retry = node_schema.metadata.retry.unwrap_or(0);
            let node_name = node_schema.metadata.name;
            let delay = node_schema.metadata.duration.unwrap_or(0) as u64;

            let handle = async move {
                emitter
                    .emit(NODE_EVENT, NodeEventPayload::running(node_id.clone()))
                    .unwrap_or_default();

                log::info!(
                    "wait_count {} {}",
                    action,
                    wait_count.load(std::sync::atomic::Ordering::SeqCst)
                );

                if wait_count.load(std::sync::atomic::Ordering::SeqCst) > 1 {
                    wait_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    log::info!(
                        "waiting for {}",
                        wait_count.load(std::sync::atomic::Ordering::SeqCst)
                    );
                    emitter
                        .emit(NODE_EVENT, NodeEventPayload::waiting(node_id.clone()))
                        .unwrap_or_default();
                    return Ok(None);
                }

                if let Some(condition) = node_schema.metadata.conditions {
                    let result = condition.check(&ctx).await?;
                    if !result.pass {
                        emitter
                            .emit(
                                NODE_EVENT,
                                NodeEventPayload::skip(node_id.clone(), Some(result)),
                            )
                            .unwrap_or_default();
                        return Ok(None);
                    }
                }

                let mut result: Result<Option<HashMap<String, serde_json::Value>>, String> =
                    Ok(None);
                if retry <= -1 {
                    // Infinite retry with a minimum interval between attempts.
                    let min_interval = Duration::from_millis(200);
                    let mut next_tick = Instant::now();
                    loop {
                        match runner
                            .run(
                                &ctx,
                                &node_name,
                                run_input.clone(),
                                node.input_schema().clone(),
                                node.output_schema(Default::default()).clone(),
                            )
                            .await
                        {
                            Ok(res) => {
                                emitter
                                    .emit(
                                        NODE_EVENT,
                                        NodeEventPayload::success(node_id.clone(), res.clone()),
                                    )
                                    .unwrap_or_default();
                                result = Ok(res);
                                break;
                            }
                            Err(_) => {
                                next_tick += min_interval;
                                let now = Instant::now();
                                if next_tick > now {
                                    sleep_until(next_tick).await;
                                } else {
                                    next_tick = now;
                                }
                            }
                        };
                    }
                } else {
                    let mut last_err = None;
                    // Total attempts = 1 (initial) + retry.
                    for attempt in 0..=retry {
                        match runner
                            .run(
                                &ctx,
                                &node_name,
                                run_input.clone(),
                                node.input_schema().clone(),
                                node.output_schema(Default::default()).clone(),
                            )
                            .await
                        {
                            Ok(res) => {
                                emitter
                                    .emit(
                                        NODE_EVENT,
                                        NodeEventPayload::success(node_id.clone(), res.clone()),
                                    )
                                    .unwrap_or_default();
                                last_err = None;
                                result = Ok(res);
                                break;
                            }
                            Err(e) => {
                                last_err = Some(e);
                                if attempt < retry {
                                    tokio::time::sleep(Duration::from_millis(delay)).await;
                                }
                            }
                        }
                    }

                    if let Some(err) = last_err {
                        emitter
                            .emit(
                                NODE_EVENT,
                                NodeEventPayload::error::<String>(node_id.clone(), None),
                            )
                            .unwrap_or_default();
                        return Err(err);
                    }
                }
                log::info!("handle finished {}", action);
                if next_node.is_empty() {
                    return result;
                }
                handle_nod(next_node, ctx, token, bus, emitter).await
            };

            tasks.spawn(async move {
                tokio::select! {
                    _ = token_clone.cancelled() => {
                        log::info!("Pipeline terminated, exiting loop");
                        emitter_clone.emit(NODE_EVENT, NodeEventPayload::cancel()).unwrap_or_default();
                        Ok(None)
                    },
                    result = handle => result
                }
            });
        }

        let mut final_result = Ok(None);

        while let Some(res) = tasks.join_next().await {
            if let Err(e) = res {
                return Err(e.to_string());
            }
            if let Ok(result) = res {
                final_result = result;
            }
        }

        final_result
    })
}

pub async fn handle_retry<F, Fut, R>(
    retry: i32,
    delay_ms: u64,
    min_interval: u64,
    mut handle: F,
) -> Result<R, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<R, String>>,
{
    let mut last_err = match handle().await {
        Ok(result) => return Ok(result),
        Err(err) => Some(err),
    };

    let min_interval = Duration::from_millis(min_interval);
    let mut next_tick = Instant::now();

    if retry <= -1 {
        loop {
            match handle().await {
                Ok(res) => return Ok(res),
                Err(err) => {
                    log::error!("{}", err);
                    next_tick += min_interval;

                    let now = Instant::now();
                    if next_tick > now {
                        sleep_until(next_tick).await;
                    } else {
                        next_tick = now;
                    }
                }
            }
        }
    } else {
        for attempt in 0..retry {
            match handle().await {
                Ok(res) => return Ok(res),
                Err(err) => {
                    last_err = Some(err);
                    if attempt + 1 < retry {
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "retry failed, unknown error".to_string()))
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::node::start::{node::StartNode, runner::StartRunnerFactory};
    use crate::types::field::SchemaField;
    use crate::types::node::{NodeRunnerControl, NodeRunnerController};
    use crate::{
        register::bus::NodeRegisterBus,
        schema::{node::Position, workflow::Connection},
        types::{
            MetaData,
            node::{I18nValue, NodeDefine, NodeRunner, NodeRunnerFactory},
        },
    };
    use serde_json::Value as JsonValue;
    use std::path::PathBuf;
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio_util::sync::CancellationToken;

    fn metadata(name: &str) -> MetaData {
        MetaData {
            name: name.to_string(),
            description: None,
            duration: None,
            retry: None,
            interval: None,
            conditions: None,
            err_return: None,
        }
    }

    struct TestRunnerFactory {
        counter: Arc<AtomicUsize>,
        params: Arc<Mutex<Option<JsonValue>>>,
    }

    impl TestRunnerFactory {
        fn new(counter: Arc<AtomicUsize>, params: Arc<Mutex<Option<JsonValue>>>) -> Self {
            Self { counter, params }
        }
    }

    impl NodeRunnerFactory for TestRunnerFactory {
        fn create(&self) -> Box<dyn NodeRunnerControl> {
            Box::new(NodeRunnerController::new(TestRunner {
                counter: Arc::clone(&self.counter),
                params: Arc::clone(&self.params),
            }))
        }
    }

    #[derive(Default)]
    struct TestNodeDefine;

    impl NodeDefine for TestNodeDefine {
        fn action_type(&self) -> String {
            "Custom".to_string()
        }

        fn name(&self) -> I18nValue {
            I18nValue {
                zh: "自定义".to_string(),
                en: "Custom".to_string(),
            }
        }

        fn icon(&self) -> String {
            String::new()
        }

        fn category(&self) -> Option<I18nValue> {
            None
        }

        fn description(&self) -> Option<I18nValue> {
            None
        }

        fn output_schema(&self, _input: HashMap<String, serde_json::Value>) -> Vec<SchemaField> {
            vec![]
        }

        fn input_schema(&self) -> Vec<SchemaField> {
            vec![]
        }
    }

    struct TestRunner {
        counter: Arc<AtomicUsize>,
        params: Arc<Mutex<Option<JsonValue>>>,
    }

    #[async_trait::async_trait]
    impl NodeRunner for TestRunner {
        type ParamType = HashMap<String, serde_json::Value>;

        async fn run(
            &mut self,
            _ctx: &Context,
            param: Self::ParamType,
        ) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
            let value: JsonValue = serde_json::to_value(param).map_err(|e| e.to_string())?;

            self.counter.fetch_add(1, Ordering::SeqCst);
            let mut guard = self
                .params
                .lock()
                .map_err(|e| format!("lock params failed: {e}"))?;
            *guard = Some(value);
            Ok(None)
        }
    }

    #[tokio::test]
    async fn run_executes_nodes_with_params() {
        let workflow = WorkflowSchema {
            nodes: vec![
                NodeSchema {
                    node_id: "node-0".to_string(),
                    action_type: "Start".to_string(),
                    metadata: metadata("start"),
                    params: None,
                    input_data: Some(HashMap::from([(
                        "params".to_string(),
                        serde_json::json!({"start_value": 1}),
                    )])),
                    position: Position::default(),
                    icon: None,
                    type_define: None,
                },
                NodeSchema {
                    node_id: "node-1".to_string(),
                    action_type: "Custom".to_string(),
                    metadata: metadata("custom"),
                    params: None,
                    input_data: Some(HashMap::from([(
                        "foo".to_string(),
                        serde_json::json!("bar"),
                    )])),
                    position: Position::default(),
                    icon: None,
                    type_define: None,
                },
            ],
            connections: vec![Connection {
                from: "node-0".to_string(),
                to: "node-1".to_string(),
            }],
        };

        let runner = WorkflowRunner::create(workflow).expect("workflow should be valid");
        let counter = Arc::new(AtomicUsize::new(0));
        let params = Arc::new(Mutex::new(None));

        let mut bus = NodeRegisterBus::new();
        bus.register(
            Box::new(StartNode::new()),
            Box::new(StartRunnerFactory::new()),
        );
        bus.register(
            Box::new(TestNodeDefine::default()),
            Box::new(TestRunnerFactory::new(
                Arc::clone(&counter),
                Arc::clone(&params),
            )),
        );

        #[cfg(feature = "tauri")]
        let context = Context::new(PathBuf::new(), None);

        #[cfg(not(feature = "tauri"))]
        let context = Context::new(PathBuf::new());

        let ctx = Arc::new(context);

        let emitter = NotificationEmitter::new();

        runner
            .run(
                ctx,
                CancellationToken::new(),
                Arc::new(RwLock::new(bus)),
                Arc::new(emitter),
            )
            .await
            .expect("workflow should run successfully");

        assert_eq!(counter.load(Ordering::SeqCst), 1);
        let stored_params = params.lock().expect("lock params failed").clone();
        assert_eq!(
            stored_params,
            Some(serde_json::json!({"foo": "bar"})),
            "runner should receive input_data"
        );
    }

    #[test]
    fn create_fails_on_cycle() {
        let workflow = WorkflowSchema {
            nodes: vec![
                NodeSchema {
                    node_id: "node-0".to_string(),
                    action_type: "Start".to_string(),
                    metadata: metadata("start"),
                    params: None,
                    input_data: None,
                    position: Position::default(),
                    icon: None,
                    type_define: None,
                },
                NodeSchema {
                    node_id: "node-1".to_string(),
                    action_type: "Custom".to_string(),
                    metadata: metadata("custom"),
                    params: None,
                    input_data: None,
                    position: Position::default(),
                    icon: None,
                    type_define: None,
                },
                NodeSchema {
                    node_id: "node-2".to_string(),
                    action_type: "Custom".to_string(),
                    metadata: metadata("custom-2"),
                    params: None,
                    input_data: None,
                    position: Position::default(),
                    icon: None,
                    type_define: None,
                },
            ],
            connections: vec![
                Connection {
                    from: "node-0".to_string(),
                    to: "node-1".to_string(),
                },
                Connection {
                    from: "node-1".to_string(),
                    to: "node-2".to_string(),
                },
                Connection {
                    from: "node-2".to_string(),
                    to: "node-1".to_string(),
                },
            ],
        };

        let err = WorkflowRunner::create(workflow).unwrap_err();
        assert!(err.contains("cycle"), "unexpected error message: {err}");
    }
}
