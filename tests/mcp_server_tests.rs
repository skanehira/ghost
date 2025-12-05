use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use ghost::app::config::Config;
use ghost::app::storage::{self, Task, TaskStatus};
use ghost::mcp::GhostServerHandler;
use rusqlite::Connection;
use rust_mcp_sdk::McpServer;
use rust_mcp_sdk::auth::AuthInfo;
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::schema::schema_utils::{ClientMessage, MessageFromServer, ServerMessage};
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolRequestParams, CallToolResult, ContentBlock, Implementation,
    InitializeRequestParams, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::RwLockReadGuard;

struct McpTestContext {
    _temp_dir: TempDir,
    config: Config,
    original_data_dir: Option<String>,
}

impl McpTestContext {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let config = Config::with_data_dir(temp_dir.path().to_path_buf());
        config
            .ensure_directories()
            .expect("failed to init directories");

        let original = std::env::var("GHOST_DATA_DIR").ok();
        unsafe {
            std::env::set_var("GHOST_DATA_DIR", config.data_dir.clone());
        }

        Self {
            _temp_dir: temp_dir,
            config,
            original_data_dir: original,
        }
    }

    fn connection(&self) -> Connection {
        storage::database::init_database_with_config(Some(self.config.clone()))
            .expect("failed to init database")
    }

    fn log_path(&self, filename: &str) -> std::path::PathBuf {
        self.config.log_dir.join(filename)
    }
}

impl Drop for McpTestContext {
    fn drop(&mut self) {
        if let Some(original) = &self.original_data_dir {
            unsafe {
                std::env::set_var("GHOST_DATA_DIR", original);
            }
        } else {
            unsafe {
                std::env::remove_var("GHOST_DATA_DIR");
            }
        }
    }
}

#[derive(Debug)]
struct DummyRuntime {
    server_info: InitializeResult,
    client_info: Mutex<Option<InitializeRequestParams>>,
}

impl Default for DummyRuntime {
    fn default() -> Self {
        Self {
            server_info: InitializeResult {
                server_info: Implementation {
                    name: "ghost-test".into(),
                    title: Some("Ghost Test Runtime".into()),
                    version: "0.0.0".into(),
                },
                capabilities: ServerCapabilities::default(),
                instructions: None,
                meta: None,
                protocol_version: LATEST_PROTOCOL_VERSION.into(),
            },
            client_info: Mutex::new(None),
        }
    }
}

#[async_trait]
impl McpServer for DummyRuntime {
    async fn start(self: Arc<Self>) -> rust_mcp_sdk::error::SdkResult<()> {
        Ok(())
    }

    async fn set_client_details(
        &self,
        client_details: InitializeRequestParams,
    ) -> rust_mcp_sdk::error::SdkResult<()> {
        *self.client_info.lock().unwrap() = Some(client_details);
        Ok(())
    }

    fn server_info(&self) -> &InitializeResult {
        &self.server_info
    }

    fn client_info(&self) -> Option<InitializeRequestParams> {
        self.client_info.lock().unwrap().clone()
    }

    async fn wait_for_initialization(&self) {}

    async fn send(
        &self,
        _message: MessageFromServer,
        _request_id: Option<rust_mcp_sdk::schema::RequestId>,
        _request_timeout: Option<Duration>,
    ) -> rust_mcp_sdk::error::SdkResult<Option<ClientMessage>> {
        Ok(None)
    }

    async fn send_batch(
        &self,
        _messages: Vec<ServerMessage>,
        _request_timeout: Option<Duration>,
    ) -> rust_mcp_sdk::error::SdkResult<Option<Vec<ClientMessage>>> {
        Ok(None)
    }

    async fn stderr_message(&self, _message: String) -> rust_mcp_sdk::error::SdkResult<()> {
        Ok(())
    }

    async fn auth_info(&self) -> RwLockReadGuard<'_, Option<AuthInfo>> {
        unimplemented!()
    }

    async fn auth_info_cloned(&self) -> Option<AuthInfo> {
        unimplemented!()
    }

    async fn update_auth_info(&self, _auth_info: Option<AuthInfo>) {
        unimplemented!()
    }
}

fn make_call_request(name: &str, args: Value) -> CallToolRequest {
    let arguments = match args {
        Value::Object(map) => map,
        Value::Null => serde_json::Map::new(),
        other => panic!("tool arguments must be an object, got {other:?}"),
    };

    CallToolRequest::new(CallToolRequestParams {
        name: name.to_string(),
        arguments: Some(arguments),
    })
}

fn insert_task_with_log(ctx: &McpTestContext, conn: &Connection, id: &str, log_contents: &str) {
    let command = vec!["echo".to_string(), "ghost".to_string()];
    let log_path = ctx.log_path(&format!("{id}.log"));
    std::fs::write(&log_path, log_contents).expect("failed to write log file");

    storage::insert_task(
        conn,
        id,
        12345,
        Some(12345),
        &command,
        None,
        None,
        &log_path,
    )
    .expect("failed to insert task");

    storage::update_task_status(conn, id, TaskStatus::Exited, Some(0))
        .expect("failed to update task status");
}

fn text_content(result: &CallToolResult) -> String {
    let block = result
        .content
        .first()
        .expect("tool result should contain content");

    match block {
        ContentBlock::TextContent(text) => text.text.clone(),
        other => panic!("unexpected content block: {other:?}"),
    }
}

async fn call_tool(handler: &GhostServerHandler, name: &str, args: Value) -> CallToolResult {
    handler
        .handle_call_tool_request(
            make_call_request(name, args),
            Arc::new(DummyRuntime::default()),
        )
        .await
        .unwrap_or_else(|_| panic!("{name} call should succeed"))
}

#[tokio::test]
async fn ghost_list_returns_all_tasks() {
    let ctx = McpTestContext::new();
    let conn = ctx.connection();

    insert_task_with_log(&ctx, &conn, "task-alpha", "alpha log");
    insert_task_with_log(&ctx, &conn, "task-beta", "beta log");

    let handler = GhostServerHandler::new(conn);
    let response = call_tool(&handler, "ghost_list", json!({})).await;

    let payload = text_content(&response);
    let mut tasks: Vec<Task> = serde_json::from_str(&payload).expect("valid task list JSON");

    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    let ids: Vec<_> = tasks.into_iter().map(|t| t.id).collect();
    assert_eq!(ids, vec!["task-alpha", "task-beta"]);
}

#[tokio::test]
async fn ghost_status_returns_requested_task() {
    let ctx = McpTestContext::new();
    let conn = ctx.connection();

    insert_task_with_log(&ctx, &conn, "task-status", "status log");

    let handler = GhostServerHandler::new(conn);
    let response = call_tool(&handler, "ghost_status", json!({ "id": "task-status" })).await;

    let payload = text_content(&response);
    let task: Task = serde_json::from_str(&payload).expect("valid task JSON");

    assert_eq!(task.id, "task-status");
    assert_eq!(task.status, TaskStatus::Exited);
}

#[tokio::test]
async fn ghost_log_returns_task_log_contents() {
    let ctx = McpTestContext::new();
    let conn = ctx.connection();

    let log_body = "line 1\nline 2\n";
    insert_task_with_log(&ctx, &conn, "task-log", log_body);

    let handler = GhostServerHandler::new(conn);
    let response = call_tool(&handler, "ghost_log", json!({ "id": "task-log" })).await;

    let payload = text_content(&response);
    assert_eq!(payload, log_body);
}

#[tokio::test]
async fn ghost_run_and_stop_lifecycle() {
    let ctx = McpTestContext::new();
    let conn = ctx.connection();
    let handler = GhostServerHandler::new(conn);

    let run_result = call_tool(
        &handler,
        "ghost_run",
        json!({
            "command": "sleep",
            "args": ["5"],
            "env": []
        }),
    )
    .await;

    let run_payload = text_content(&run_result);
    let task: Task = serde_json::from_str(&run_payload).expect("valid run task JSON");
    let task_id = task.id.clone();
    assert_eq!(task.status, TaskStatus::Running);
    assert!(task.command.contains("sleep"));

    assert!(!task.log_path.is_empty());

    tokio::time::sleep(Duration::from_millis(100)).await;

    let stop_id = task_id.clone();

    let stop_result = call_tool(
        &handler,
        "ghost_stop",
        json!({
            "id": stop_id,
            "force": true
        }),
    )
    .await;

    let stop_message = text_content(&stop_result);
    assert!(stop_message.contains("stopped successfully"));

    tokio::time::sleep(Duration::from_millis(100)).await;

    let status_result = call_tool(&handler, "ghost_status", json!({ "id": task_id })).await;

    let status_payload = text_content(&status_result);
    let updated_task: Task = serde_json::from_str(&status_payload).expect("valid status task JSON");
    assert_eq!(updated_task.status, TaskStatus::Killed);
}
