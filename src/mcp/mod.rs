use async_trait::async_trait;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::mcp_server::{ServerHandler, server_runtime};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, Implementation, InitializeResult, LATEST_PROTOCOL_VERSION,
    ListToolsRequest, ListToolsResult, ServerCapabilities, ServerCapabilitiesTools, TextContent,
};
use rust_mcp_sdk::{McpServer, tool_box};
use rust_mcp_transport::{StdioTransport, TransportOptions};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::commands;
use crate::app::storage::task_repository;
use rusqlite::Connection;

#[mcp_tool(
    name = "ghost_run",
    description = "Run a command as a background process managed by ghost"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct RunTool {
    /// Command to run
    pub command: String,
    /// Optional command arguments
    pub args: Option<Vec<String>>,
    /// Working directory (defaults to current directory)
    pub cwd: Option<String>,
    /// Environment variables
    pub env: Option<Vec<String>>,
}

#[mcp_tool(
    name = "ghost_list",
    description = "List all processes managed by ghost"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct ListTool {
    /// Filter by status (running, stopped, failed)
    pub status: Option<String>,
}

#[mcp_tool(name = "ghost_stop", description = "Stop a running process by ID")]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct StopTool {
    /// Process ID to stop
    pub id: String,
    /// Force kill the process (SIGKILL instead of SIGTERM)
    pub force: Option<bool>,
}

#[mcp_tool(name = "ghost_log", description = "Get logs for a specific process")]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct LogTool {
    /// Process ID to get logs for
    pub id: String,
}

#[mcp_tool(
    name = "ghost_status",
    description = "Check status of a specific process"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct StatusTool {
    /// Process ID to check status for
    pub id: String,
}

tool_box!(
    GhostTools,
    [RunTool, ListTool, StopTool, LogTool, StatusTool]
);

pub struct GhostServerHandler {
    conn: Arc<Mutex<Connection>>,
}

impl GhostServerHandler {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

#[async_trait]
impl ServerHandler for GhostServerHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
        Ok(ListToolsResult {
            tools: GhostTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let params = request.params.clone();
        let tool = GhostTools::try_from(params)?;

        match tool {
            GhostTools::RunTool(t) => {
                let mut command = vec![t.command];
                if let Some(args) = t.args {
                    command.extend(args);
                }

                let cwd = t.cwd.map(PathBuf::from);
                let env = t.env.unwrap_or_default();

                let conn = self.conn.lock().unwrap();
                let process_info = commands::spawn(&conn, command, cwd, env, false)
                    .map_err(|e| CallToolError::from_message(format!("Failed to run: {e}")))?;

                // Get the task from database to return complete info
                let task = task_repository::get_task(&conn, &process_info.id)
                    .map_err(|e| CallToolError::from_message(format!("Failed to get task: {e}")))?;

                let result = serde_json::to_string_pretty(&task)
                    .map_err(|e| CallToolError::from_message(format!("JSON error: {e}")))?;

                Ok(CallToolResult::text_content(vec![TextContent::new(
                    result, None, None,
                )]))
            }
            GhostTools::ListTool(t) => {
                let conn = self.conn.lock().unwrap();

                // Prepare status filter
                let tasks = commands::list(&conn, t.status, false).map_err(|e| {
                    CallToolError::from_message(format!("Failed to list tasks: {e}"))
                })?;

                let result = serde_json::to_string_pretty(&tasks)
                    .map_err(|e| CallToolError::from_message(format!("JSON error: {e}")))?;

                Ok(CallToolResult::text_content(vec![TextContent::new(
                    result, None, None,
                )]))
            }
            GhostTools::StopTool(t) => {
                let conn = self.conn.lock().unwrap();
                commands::stop(&conn, &t.id, t.force.unwrap_or(false), false)
                    .map_err(|e| CallToolError::from_message(format!("Failed to stop: {e}")))?;

                Ok(CallToolResult::text_content(vec![TextContent::new(
                    format!("Process {} stopped successfully", t.id),
                    None,
                    None,
                )]))
            }
            GhostTools::LogTool(t) => {
                let conn = self.conn.lock().unwrap();
                let task = task_repository::get_task(&conn, &t.id)
                    .map_err(|e| CallToolError::from_message(format!("Failed to get task: {e}")))?;

                let log_content = std::fs::read_to_string(&task.log_path)
                    .map_err(|e| CallToolError::from_message(format!("Failed to read log: {e}")))?;

                Ok(CallToolResult::text_content(vec![TextContent::new(
                    log_content,
                    None,
                    None,
                )]))
            }
            GhostTools::StatusTool(t) => {
                let conn = self.conn.lock().unwrap();
                let task = commands::status(&conn, &t.id, false).map_err(|e| {
                    CallToolError::from_message(format!("Failed to get status: {e}"))
                })?;

                let result = serde_json::to_string_pretty(&task)
                    .map_err(|e| CallToolError::from_message(format!("JSON error: {e}")))?;

                Ok(CallToolResult::text_content(vec![TextContent::new(
                    result, None, None,
                )]))
            }
        }
    }
}

pub async fn run_stdio_server(conn: Connection) -> Result<(), Box<dyn std::error::Error>> {
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "ghost-mcp".into(),
            title: Some("Ghost MCP Server".into()),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Ghost MCP server for managing background processes. Use tools to run, list, stop, check status, cleanup old tasks, and view logs for processes.".into()
        ),
        protocol_version: LATEST_PROTOCOL_VERSION.into(),
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = GhostServerHandler::new(conn);
    let server = server_runtime::create_server(server_details, transport, handler);
    server.start().await?;
    Ok(())
}
