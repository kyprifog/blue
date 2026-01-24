//! MCP Server implementation
//!
//! Handles JSON-RPC requests and routes to appropriate tool handlers.

use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

use blue_core::{detect_blue, DocType, Document, ProjectState, Rfc};

use crate::error::ServerError;

/// Blue MCP Server state
pub struct BlueServer {
    /// Current working directory
    cwd: Option<PathBuf>,
    /// Cached project state
    state: Option<ProjectState>,
}

impl BlueServer {
    pub fn new() -> Self {
        Self {
            cwd: None,
            state: None,
        }
    }

    /// Try to load project state for the current directory
    fn ensure_state(&mut self) -> Result<&ProjectState, ServerError> {
        if self.state.is_none() {
            let cwd = self.cwd.as_ref().ok_or(ServerError::BlueNotDetected)?;
            let home = detect_blue(cwd).map_err(|_| ServerError::BlueNotDetected)?;

            // Try to get project name from the current path
            let project = home.project_name.clone().unwrap_or_else(|| "default".to_string());

            let state = ProjectState::load(home, &project)
                .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

            self.state = Some(state);
        }

        self.state.as_ref().ok_or(ServerError::BlueNotDetected)
    }

    /// Handle a JSON-RPC request
    pub fn handle_request(&mut self, request: &str) -> String {
        let result = self.handle_request_inner(request);
        match result {
            Ok(response) => response,
            Err(e) => {
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": e.code(),
                        "message": e.to_string()
                    },
                    "id": null
                });
                serde_json::to_string(&error_response).unwrap_or_default()
            }
        }
    }

    fn handle_request_inner(&mut self, request: &str) -> Result<String, ServerError> {
        let req: JsonRpcRequest = serde_json::from_str(request)?;

        debug!("Received request: {} (id: {:?})", req.method, req.id);

        let result = match req.method.as_str() {
            "initialize" => self.handle_initialize(&req.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tool_call(&req.params),
            _ => Err(ServerError::MethodNotFound(req.method.clone())),
        };

        let response = match result {
            Ok(value) => json!({
                "jsonrpc": "2.0",
                "result": value,
                "id": req.id
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": e.code(),
                    "message": e.to_string()
                },
                "id": req.id
            }),
        };

        Ok(serde_json::to_string(&response)?)
    }

    /// Handle initialize request
    fn handle_initialize(&mut self, _params: &Option<Value>) -> Result<Value, ServerError> {
        info!("MCP initialize");
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "blue",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    /// Handle tools/list request
    fn handle_tools_list(&self) -> Result<Value, ServerError> {
        Ok(json!({
            "tools": [
                {
                    "name": "blue_status",
                    "description": "Get project status. Returns active work, ready items, stalled items, and recommendations.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            }
                        }
                    }
                },
                {
                    "name": "blue_next",
                    "description": "Get recommended next actions based on project state.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            }
                        }
                    }
                },
                {
                    "name": "blue_rfc_create",
                    "description": "Create a new RFC (design document) for a feature.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title in kebab-case"
                            },
                            "problem": {
                                "type": "string",
                                "description": "Problem statement or summary"
                            },
                            "source_spike": {
                                "type": "string",
                                "description": "Source spike title that led to this RFC"
                            }
                        },
                        "required": ["title"]
                    }
                },
                {
                    "name": "blue_rfc_get",
                    "description": "Get an RFC by title or number.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title or number"
                            }
                        },
                        "required": ["title"]
                    }
                },
                {
                    "name": "blue_rfc_update_status",
                    "description": "Update an RFC's status (draft -> accepted -> in-progress -> implemented).",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title"
                            },
                            "status": {
                                "type": "string",
                                "description": "New status: accepted, in-progress, implemented, or superseded",
                                "enum": ["accepted", "in-progress", "implemented", "superseded"]
                            }
                        },
                        "required": ["title", "status"]
                    }
                },
                {
                    "name": "blue_rfc_plan",
                    "description": "Create or update an implementation plan with checkboxes for an RFC.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title"
                            },
                            "tasks": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "List of implementation tasks"
                            }
                        },
                        "required": ["title", "tasks"]
                    }
                },
                {
                    "name": "blue_rfc_task_complete",
                    "description": "Mark a task as complete in an RFC plan.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title"
                            },
                            "task": {
                                "type": "string",
                                "description": "Task index (1-based) or substring to match"
                            }
                        },
                        "required": ["title", "task"]
                    }
                },
                {
                    "name": "blue_rfc_validate",
                    "description": "Check RFC status and plan completion.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "title": {
                                "type": "string",
                                "description": "RFC title"
                            }
                        },
                        "required": ["title"]
                    }
                },
                {
                    "name": "blue_search",
                    "description": "Search documents using full-text search.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "cwd": {
                                "type": "string",
                                "description": "Current working directory"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "doc_type": {
                                "type": "string",
                                "description": "Filter by document type",
                                "enum": ["rfc", "spike", "adr", "decision"]
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum results to return (default: 10)"
                            }
                        },
                        "required": ["query"]
                    }
                }
            ]
        }))
    }

    /// Handle tools/call request
    fn handle_tool_call(&mut self, params: &Option<Value>) -> Result<Value, ServerError> {
        let params = params.as_ref().ok_or(ServerError::InvalidParams)?;
        let call: ToolCallParams = serde_json::from_value(params.clone())?;

        // Extract cwd from arguments if present
        if let Some(ref args) = call.arguments {
            if let Some(cwd) = args.get("cwd").and_then(|v| v.as_str()) {
                self.cwd = Some(PathBuf::from(cwd));
                // Reset state when cwd changes
                self.state = None;
            }
        }

        let result = match call.name.as_str() {
            "blue_status" => self.handle_status(&call.arguments),
            "blue_next" => self.handle_next(&call.arguments),
            "blue_rfc_create" => self.handle_rfc_create(&call.arguments),
            "blue_rfc_get" => self.handle_rfc_get(&call.arguments),
            "blue_rfc_update_status" => self.handle_rfc_update_status(&call.arguments),
            "blue_rfc_plan" => self.handle_rfc_plan(&call.arguments),
            "blue_rfc_task_complete" => self.handle_rfc_task_complete(&call.arguments),
            "blue_rfc_validate" => self.handle_rfc_validate(&call.arguments),
            "blue_search" => self.handle_search(&call.arguments),
            _ => Err(ServerError::ToolNotFound(call.name)),
        }?;

        // Wrap result in MCP tool call response format
        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&result)?
            }]
        }))
    }

    fn handle_status(&mut self, _args: &Option<Value>) -> Result<Value, ServerError> {
        match self.ensure_state() {
            Ok(state) => {
                let summary = state.status_summary();
                Ok(json!({
                    "active": summary.active,
                    "ready": summary.ready,
                    "stalled": summary.stalled,
                    "drafts": summary.drafts,
                    "hint": summary.hint
                }))
            }
            Err(_) => {
                // Fall back to a simple message if not in a Blue project
                Ok(json!({
                    "message": blue_core::voice::error(
                        "Can't find Blue here",
                        "Run 'blue init' to set up this project"
                    ),
                    "active": [],
                    "ready": [],
                    "stalled": [],
                    "drafts": []
                }))
            }
        }
    }

    fn handle_next(&mut self, _args: &Option<Value>) -> Result<Value, ServerError> {
        match self.ensure_state() {
            Ok(state) => {
                let summary = state.status_summary();

                let recommendations = if !summary.stalled.is_empty() {
                    vec![format!(
                        "'{}' might be stalled. Check if work is still in progress.",
                        summary.stalled[0].title
                    )]
                } else if !summary.ready.is_empty() {
                    vec![format!(
                        "'{}' is ready to implement. Run 'blue worktree create {}' to start.",
                        summary.ready[0].title, summary.ready[0].title
                    )]
                } else if !summary.drafts.is_empty() {
                    vec![format!(
                        "'{}' is in draft. Review and accept it when ready.",
                        summary.drafts[0].title
                    )]
                } else if !summary.active.is_empty() {
                    vec![format!(
                        "{} item(s) in progress. Keep at it.",
                        summary.active.len()
                    )]
                } else {
                    vec!["Nothing pressing. Good time to plan something new.".to_string()]
                };

                Ok(json!({
                    "recommendations": recommendations,
                    "hint": summary.hint
                }))
            }
            Err(_) => {
                Ok(json!({
                    "recommendations": [
                        "Run 'blue init' to set up this project first."
                    ],
                    "hint": "Can't find Blue here."
                }))
            }
        }
    }

    fn handle_rfc_create(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let problem = args.get("problem").and_then(|v| v.as_str());
        let source_spike = args.get("source_spike").and_then(|v| v.as_str());

        match self.ensure_state() {
            Ok(state) => {
                // Get next RFC number
                let number = state.store.next_number(DocType::Rfc)
                    .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

                // Create document in store
                let mut doc = Document::new(DocType::Rfc, title, "draft");
                doc.number = Some(number);

                let id = state.store.add_document(&doc)
                    .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

                // Generate markdown
                let mut rfc = Rfc::new(title);
                if let Some(p) = problem {
                    rfc.problem = Some(p.to_string());
                }
                if let Some(s) = source_spike {
                    rfc.source_spike = Some(s.to_string());
                }

                let markdown = rfc.to_markdown(number as u32);

                Ok(json!({
                    "status": "success",
                    "id": id,
                    "number": number,
                    "title": title,
                    "markdown": markdown,
                    "message": blue_core::voice::success(
                        &format!("Created RFC {:04}: '{}'", number, title),
                        Some("Want me to help fill in the details?")
                    )
                }))
            }
            Err(_) => {
                // Create RFC without persistence (just generate markdown)
                let rfc = Rfc::new(title);
                let markdown = rfc.to_markdown(1);

                Ok(json!({
                    "status": "success",
                    "number": 1,
                    "title": title,
                    "markdown": markdown,
                    "message": blue_core::voice::success(
                        &format!("Created RFC '{}'", title),
                        Some("Note: Not persisted - run 'blue init' to enable storage.")
                    )
                }))
            }
        }
    }

    fn handle_rfc_get(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let state = self.ensure_state()?;

        let doc = state.store.find_document(DocType::Rfc, title)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        // Get tasks if any
        let tasks = if let Some(id) = doc.id {
            state.store.get_tasks(id).unwrap_or_default()
        } else {
            vec![]
        };

        let progress = if let Some(id) = doc.id {
            state.store.get_task_progress(id).ok()
        } else {
            None
        };

        Ok(json!({
            "id": doc.id,
            "number": doc.number,
            "title": doc.title,
            "status": doc.status,
            "file_path": doc.file_path,
            "created_at": doc.created_at,
            "updated_at": doc.updated_at,
            "tasks": tasks.iter().map(|t| json!({
                "index": t.task_index,
                "description": t.description,
                "completed": t.completed
            })).collect::<Vec<_>>(),
            "progress": progress.map(|p| json!({
                "completed": p.completed,
                "total": p.total,
                "percentage": p.percentage
            }))
        }))
    }

    fn handle_rfc_update_status(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let state = self.ensure_state()?;

        state.store.update_document_status(DocType::Rfc, title, status)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "title": title,
            "new_status": status,
            "message": blue_core::voice::success(
                &format!("Updated '{}' to {}", title, status),
                None
            )
        }))
    }

    fn handle_rfc_plan(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let tasks: Vec<String> = args
            .get("tasks")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let state = self.ensure_state()?;

        let doc = state.store.find_document(DocType::Rfc, title)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        let doc_id = doc.id.ok_or(ServerError::InvalidParams)?;

        state.store.set_tasks(doc_id, &tasks)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "title": title,
            "task_count": tasks.len(),
            "message": blue_core::voice::success(
                &format!("Set {} tasks for '{}'", tasks.len(), title),
                Some("Mark them complete as you go with blue_rfc_task_complete.")
            )
        }))
    }

    fn handle_rfc_task_complete(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let state = self.ensure_state()?;

        let doc = state.store.find_document(DocType::Rfc, title)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        let doc_id = doc.id.ok_or(ServerError::InvalidParams)?;

        // Parse task index or find by substring
        let task_index = if let Ok(idx) = task.parse::<i32>() {
            idx
        } else {
            // Find task by substring
            let tasks = state.store.get_tasks(doc_id)
                .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

            tasks.iter()
                .find(|t| t.description.to_lowercase().contains(&task.to_lowercase()))
                .map(|t| t.task_index)
                .ok_or(ServerError::InvalidParams)?
        };

        state.store.complete_task(doc_id, task_index)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        let progress = state.store.get_task_progress(doc_id)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        Ok(json!({
            "status": "success",
            "title": title,
            "task_index": task_index,
            "progress": {
                "completed": progress.completed,
                "total": progress.total,
                "percentage": progress.percentage
            },
            "message": blue_core::voice::success(
                &format!("Task {} complete. {} of {} done ({}%)",
                    task_index, progress.completed, progress.total, progress.percentage),
                None
            )
        }))
    }

    fn handle_rfc_validate(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let state = self.ensure_state()?;

        let doc = state.store.find_document(DocType::Rfc, title)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        let doc_id = doc.id.ok_or(ServerError::InvalidParams)?;

        let progress = state.store.get_task_progress(doc_id)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        let message = if progress.total == 0 {
            "No plan defined yet. Use blue_rfc_plan to add tasks.".to_string()
        } else if progress.percentage == 100 {
            format!("All {} tasks complete. Ready to mark as implemented.", progress.total)
        } else if progress.percentage >= 70 {
            format!("{}% done ({}/{}). Getting close.", progress.percentage, progress.completed, progress.total)
        } else {
            format!("{}% done ({}/{}). Keep going.", progress.percentage, progress.completed, progress.total)
        };

        Ok(json!({
            "title": doc.title,
            "status": doc.status,
            "progress": {
                "completed": progress.completed,
                "total": progress.total,
                "percentage": progress.percentage
            },
            "message": message
        }))
    }

    fn handle_search(&mut self, args: &Option<Value>) -> Result<Value, ServerError> {
        let args = args.as_ref().ok_or(ServerError::InvalidParams)?;

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or(ServerError::InvalidParams)?;

        let doc_type = args.get("doc_type").and_then(|v| v.as_str()).and_then(DocType::from_str);
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let state = self.ensure_state()?;

        let results = state.store.search_documents(query, doc_type, limit)
            .map_err(|e| ServerError::StateLoadFailed(e.to_string()))?;

        Ok(json!({
            "query": query,
            "count": results.len(),
            "results": results.iter().map(|r| json!({
                "title": r.document.title,
                "type": r.document.doc_type.as_str(),
                "status": r.document.status,
                "score": r.score
            })).collect::<Vec<_>>()
        }))
    }
}

impl Default for BlueServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Option<Value>,
}
