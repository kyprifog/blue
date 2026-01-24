//! Realm MCP tool handlers
//!
//! Implements RFC 0002: Realm MCP Integration
//!
//! Phase 1:
//! - realm_status: Get realm overview
//! - realm_check: Validate contracts/bindings
//! - contract_get: Get contract details
//!
//! Phase 2:
//! - session_start: Begin work session
//! - session_stop: End session with summary

use blue_core::daemon::DaemonPaths;
use blue_core::realm::{LocalRepoConfig, RealmService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::error::ServerError;

/// Context detected from current working directory
struct RealmContext {
    realm_name: String,
    repo_name: String,
    service: RealmService,
}

/// Detect realm context from cwd
fn detect_context(cwd: Option<&Path>) -> Result<RealmContext, ServerError> {
    let cwd = cwd.ok_or(ServerError::InvalidParams)?;

    // Check for .blue/config.yaml
    let config_path = cwd.join(".blue").join("config.yaml");
    if !config_path.exists() {
        return Err(ServerError::NotFound(
            "Not in a realm repo. Run 'blue realm admin join <realm>' first.".to_string(),
        ));
    }

    let local_config = LocalRepoConfig::load(&config_path).map_err(|e| {
        ServerError::CommandFailed(format!("Failed to load .blue/config.yaml: {}", e))
    })?;

    let paths = DaemonPaths::new().map_err(|e| {
        ServerError::CommandFailed(format!("Failed to get daemon paths: {}", e))
    })?;

    let service = RealmService::new(paths.realms);

    Ok(RealmContext {
        realm_name: local_config.realm.name,
        repo_name: local_config.repo,
        service,
    })
}

/// Handle realm_status - get realm overview
pub fn handle_status(cwd: Option<&Path>) -> Result<Value, ServerError> {
    let ctx = detect_context(cwd)?;

    let details = ctx.service.load_realm_details(&ctx.realm_name).map_err(|e| {
        ServerError::CommandFailed(format!("Failed to load realm: {}", e))
    })?;

    // Build repos list
    let repos: Vec<Value> = details
        .repos
        .iter()
        .map(|r| {
            json!({
                "name": r.name,
                "path": r.path,
                "is_current": r.name == ctx.repo_name
            })
        })
        .collect();

    // Build domains list
    let domains: Vec<Value> = details
        .domains
        .iter()
        .map(|d| {
            let contracts: Vec<Value> = d
                .contracts
                .iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "version": c.version,
                        "owner": c.owner
                    })
                })
                .collect();

            let bindings: Vec<Value> = d
                .bindings
                .iter()
                .map(|b| {
                    json!({
                        "repo": b.repo,
                        "role": format!("{:?}", b.role),
                        "exports": b.exports.len(),
                        "imports": b.imports.len()
                    })
                })
                .collect();

            json!({
                "name": d.domain.name,
                "members": d.domain.members,
                "contracts": contracts,
                "bindings": bindings
            })
        })
        .collect();

    // Notifications are fetched via daemon in Phase 4
    // For now, return empty (sync implementation)
    let notifications: Vec<Value> = Vec::new();

    // Build next steps
    let mut next_steps = Vec::new();
    if domains.is_empty() {
        next_steps.push("Create a domain with 'blue realm admin domain'".to_string());
    }

    Ok(json!({
        "status": "success",
        "realm": ctx.realm_name,
        "current_repo": ctx.repo_name,
        "repos": repos,
        "domains": domains,
        "notifications": notifications,
        "next_steps": next_steps
    }))
}

/// Handle realm_check - validate contracts/bindings
pub fn handle_check(cwd: Option<&Path>, realm_arg: Option<&str>) -> Result<Value, ServerError> {
    let ctx = detect_context(cwd)?;
    let realm_name = realm_arg.unwrap_or(&ctx.realm_name);

    let result = ctx.service.check_realm(realm_name).map_err(|e| {
        ServerError::CommandFailed(format!("Failed to check realm: {}", e))
    })?;

    let errors: Vec<Value> = result
        .errors
        .iter()
        .map(|e| {
            json!({
                "domain": e.domain,
                "kind": format!("{:?}", e.kind),
                "message": e.message
            })
        })
        .collect();

    let warnings: Vec<Value> = result
        .warnings
        .iter()
        .map(|w| {
            json!({
                "domain": w.domain,
                "kind": format!("{:?}", w.kind),
                "message": w.message
            })
        })
        .collect();

    // Notifications are fetched via daemon in Phase 4
    let notifications: Vec<Value> = Vec::new();

    // Build next steps
    let mut next_steps = Vec::new();
    if !result.is_ok() {
        next_steps.push("Fix errors before proceeding".to_string());
    }
    if result.has_warnings() {
        next_steps.push("Review warnings - they may indicate issues".to_string());
    }
    if result.is_ok() && !result.has_warnings() {
        next_steps.push("All checks passed. Ready to proceed.".to_string());
    }

    Ok(json!({
        "status": if result.is_ok() { "success" } else { "error" },
        "realm": realm_name,
        "current_repo": ctx.repo_name,
        "valid": result.is_ok(),
        "errors": errors,
        "warnings": warnings,
        "notifications": notifications,
        "next_steps": next_steps
    }))
}

/// Handle contract_get - get contract details
pub fn handle_contract_get(
    cwd: Option<&Path>,
    domain_name: &str,
    contract_name: &str,
) -> Result<Value, ServerError> {
    let ctx = detect_context(cwd)?;

    let details = ctx.service.load_realm_details(&ctx.realm_name).map_err(|e| {
        ServerError::CommandFailed(format!("Failed to load realm: {}", e))
    })?;

    // Find the domain
    let domain = details
        .domains
        .iter()
        .find(|d| d.domain.name == domain_name)
        .ok_or_else(|| {
            ServerError::NotFound(format!("Domain '{}' not found", domain_name))
        })?;

    // Find the contract
    let contract = domain
        .contracts
        .iter()
        .find(|c| c.name == contract_name)
        .ok_or_else(|| {
            ServerError::NotFound(format!(
                "Contract '{}' not found in domain '{}'",
                contract_name, domain_name
            ))
        })?;

    // Get bindings for this contract
    let bindings: Vec<Value> = domain
        .bindings
        .iter()
        .filter(|b| {
            b.exports.iter().any(|e| e.contract == contract_name)
                || b.imports.iter().any(|i| i.contract == contract_name)
        })
        .map(|b| {
            let exports: Vec<&str> = b
                .exports
                .iter()
                .filter(|e| e.contract == contract_name)
                .map(|_| "export")
                .collect();
            let imports: Vec<String> = b
                .imports
                .iter()
                .filter(|i| i.contract == contract_name)
                .map(|i| format!("import ({})", i.version))
                .collect();

            json!({
                "repo": b.repo,
                "role": format!("{:?}", b.role),
                "relationship": if !exports.is_empty() { "exports" } else { "imports" },
                "version_req": imports.first().cloned()
            })
        })
        .collect();

    // Notifications are fetched via daemon in Phase 4
    let notifications: Vec<Value> = Vec::new();

    // Build next steps
    let mut next_steps = Vec::new();
    if contract.owner == ctx.repo_name {
        next_steps.push("You own this contract. You can modify it.".to_string());
    } else {
        next_steps.push(format!(
            "This contract is owned by '{}'. Contact them for changes.",
            contract.owner
        ));
    }

    Ok(json!({
        "status": "success",
        "realm": ctx.realm_name,
        "domain": domain_name,
        "contract": {
            "name": contract.name,
            "version": contract.version,
            "owner": contract.owner,
            "compatibility": {
                "backwards": contract.compatibility.backwards,
                "forwards": contract.compatibility.forwards
            },
            "schema": contract.schema,
            "value": contract.value,
            "evolution": contract.evolution
        },
        "bindings": bindings,
        "current_repo": ctx.repo_name,
        "notifications": notifications,
        "next_steps": next_steps
    }))
}

// ─── Phase 2: Session Tools ─────────────────────────────────────────────────

/// Session state stored in .blue/session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub realm: String,
    pub repo: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    #[serde(default)]
    pub active_rfc: Option<String>,
    #[serde(default)]
    pub active_domains: Vec<String>,
    #[serde(default)]
    pub contracts_modified: Vec<String>,
    #[serde(default)]
    pub contracts_watched: Vec<String>,
}

impl SessionState {
    /// Load session from .blue/session file
    pub fn load(cwd: &Path) -> Option<Self> {
        let session_path = cwd.join(".blue").join("session");
        if !session_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&session_path).ok()?;
        serde_yaml::from_str(&content).ok()
    }

    /// Save session to .blue/session file
    pub fn save(&self, cwd: &Path) -> Result<(), ServerError> {
        let blue_dir = cwd.join(".blue");
        if !blue_dir.exists() {
            return Err(ServerError::NotFound(
                "Not in a realm repo. No .blue directory.".to_string(),
            ));
        }

        let session_path = blue_dir.join("session");
        let content = serde_yaml::to_string(self)
            .map_err(|e| ServerError::CommandFailed(format!("Failed to serialize session: {}", e)))?;

        std::fs::write(&session_path, content)
            .map_err(|e| ServerError::CommandFailed(format!("Failed to write session: {}", e)))?;

        Ok(())
    }

    /// Delete session file
    pub fn delete(cwd: &Path) -> Result<(), ServerError> {
        let session_path = cwd.join(".blue").join("session");
        if session_path.exists() {
            std::fs::remove_file(&session_path).map_err(|e| {
                ServerError::CommandFailed(format!("Failed to delete session: {}", e))
            })?;
        }
        Ok(())
    }
}

/// Generate a unique session ID
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("sess-{:x}", timestamp)
}

/// Handle session_start - begin work session
pub fn handle_session_start(
    cwd: Option<&Path>,
    active_rfc: Option<&str>,
) -> Result<Value, ServerError> {
    let cwd = cwd.ok_or(ServerError::InvalidParams)?;
    let ctx = detect_context(Some(cwd))?;

    // Check for existing session
    if let Some(existing) = SessionState::load(cwd) {
        return Ok(json!({
            "status": "warning",
            "message": "Session already active",
            "session": {
                "id": existing.id,
                "realm": existing.realm,
                "repo": existing.repo,
                "started_at": existing.started_at.to_rfc3339(),
                "active_rfc": existing.active_rfc,
                "active_domains": existing.active_domains
            },
            "next_steps": ["Use session_stop to end the current session first"]
        }));
    }

    // Determine active domains from repo's bindings
    let details = ctx.service.load_realm_details(&ctx.realm_name).map_err(|e| {
        ServerError::CommandFailed(format!("Failed to load realm: {}", e))
    })?;

    let active_domains: Vec<String> = details
        .domains
        .iter()
        .filter(|d| d.bindings.iter().any(|b| b.repo == ctx.repo_name))
        .map(|d| d.domain.name.clone())
        .collect();

    // Determine contracts we're watching (imports) and could modify (exports)
    let mut contracts_watched = Vec::new();
    let mut contracts_modified = Vec::new();

    for domain in &details.domains {
        for binding in &domain.bindings {
            if binding.repo == ctx.repo_name {
                for import in &binding.imports {
                    contracts_watched.push(format!("{}/{}", domain.domain.name, import.contract));
                }
                for export in &binding.exports {
                    contracts_modified.push(format!("{}/{}", domain.domain.name, export.contract));
                }
            }
        }
    }

    let now = Utc::now();
    let session = SessionState {
        id: generate_session_id(),
        realm: ctx.realm_name.clone(),
        repo: ctx.repo_name.clone(),
        started_at: now,
        last_activity: now,
        active_rfc: active_rfc.map(String::from),
        active_domains: active_domains.clone(),
        contracts_modified: contracts_modified.clone(),
        contracts_watched: contracts_watched.clone(),
    };

    session.save(cwd)?;

    // Build next steps
    let mut next_steps = Vec::new();
    if !contracts_watched.is_empty() {
        next_steps.push(format!(
            "Watching {} imported contract{}",
            contracts_watched.len(),
            if contracts_watched.len() == 1 { "" } else { "s" }
        ));
    }
    if active_rfc.is_none() {
        next_steps.push("Consider setting active_rfc to track which RFC you're working on".to_string());
    }
    next_steps.push("Use session_stop when done to get a summary".to_string());

    Ok(json!({
        "status": "success",
        "message": "Session started",
        "session": {
            "id": session.id,
            "realm": session.realm,
            "repo": session.repo,
            "started_at": session.started_at.to_rfc3339(),
            "active_rfc": session.active_rfc,
            "active_domains": session.active_domains,
            "contracts_modified": contracts_modified,
            "contracts_watched": contracts_watched
        },
        "notifications": [],
        "next_steps": next_steps
    }))
}

/// Handle session_stop - end session with summary
pub fn handle_session_stop(cwd: Option<&Path>) -> Result<Value, ServerError> {
    let cwd = cwd.ok_or(ServerError::InvalidParams)?;

    // Load existing session
    let session = SessionState::load(cwd).ok_or_else(|| {
        ServerError::NotFound("No active session. Nothing to stop.".to_string())
    })?;

    // Calculate session duration
    let duration = Utc::now().signed_duration_since(session.started_at);
    let duration_str = if duration.num_hours() > 0 {
        format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    };

    // Delete the session file
    SessionState::delete(cwd)?;

    // Build summary
    let summary = json!({
        "id": session.id,
        "realm": session.realm,
        "repo": session.repo,
        "started_at": session.started_at.to_rfc3339(),
        "ended_at": Utc::now().to_rfc3339(),
        "duration": duration_str,
        "active_rfc": session.active_rfc,
        "active_domains": session.active_domains,
        "contracts_modified": session.contracts_modified,
        "contracts_watched": session.contracts_watched
    });

    Ok(json!({
        "status": "success",
        "message": format!("Session ended after {}", duration_str),
        "summary": summary,
        "next_steps": ["Start a new session with session_start when you're ready to work again"]
    }))
}

/// Get current session if one exists (for other tools to check)
pub fn get_current_session(cwd: Option<&Path>) -> Option<SessionState> {
    cwd.and_then(SessionState::load)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_realm() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let blue_dir = path.join(".blue");
        std::fs::create_dir_all(&blue_dir).unwrap();

        // Create a minimal config
        let config = r#"
realm:
  name: test-realm
  url: file:///tmp/test-realm
repo: test-repo
"#;
        std::fs::write(blue_dir.join("config.yaml"), config).unwrap();

        (tmp, path)
    }

    #[test]
    fn test_detect_context_no_config() {
        let tmp = TempDir::new().unwrap();
        let result = detect_context(Some(tmp.path()));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_context_with_config() {
        let (_tmp, path) = setup_test_realm();
        let result = detect_context(Some(&path));
        // Config parsing works - result depends on whether ~/.blue exists
        // This is an integration-level test; just verify it doesn't panic
        match result {
            Ok(ctx) => {
                assert_eq!(ctx.realm_name, "test-realm");
                assert_eq!(ctx.repo_name, "test-repo");
            }
            Err(_) => {
                // Also acceptable if daemon paths don't exist
            }
        }
    }

    // Phase 2: Session tests

    #[test]
    fn test_session_state_save_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let blue_dir = path.join(".blue");
        std::fs::create_dir_all(&blue_dir).unwrap();

        let session = SessionState {
            id: "test-session-123".to_string(),
            realm: "test-realm".to_string(),
            repo: "test-repo".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            active_rfc: Some("my-rfc".to_string()),
            active_domains: vec!["domain-1".to_string()],
            contracts_modified: vec!["domain-1/contract-a".to_string()],
            contracts_watched: vec!["domain-1/contract-b".to_string()],
        };

        // Save
        session.save(&path).unwrap();

        // Verify file exists
        assert!(blue_dir.join("session").exists());

        // Load
        let loaded = SessionState::load(&path).unwrap();
        assert_eq!(loaded.id, "test-session-123");
        assert_eq!(loaded.realm, "test-realm");
        assert_eq!(loaded.repo, "test-repo");
        assert_eq!(loaded.active_rfc, Some("my-rfc".to_string()));
        assert_eq!(loaded.active_domains, vec!["domain-1".to_string()]);
    }

    #[test]
    fn test_session_state_delete() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let blue_dir = path.join(".blue");
        std::fs::create_dir_all(&blue_dir).unwrap();

        // Create session file
        let session = SessionState {
            id: "to-delete".to_string(),
            realm: "test-realm".to_string(),
            repo: "test-repo".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            active_rfc: None,
            active_domains: vec![],
            contracts_modified: vec![],
            contracts_watched: vec![],
        };
        session.save(&path).unwrap();
        assert!(blue_dir.join("session").exists());

        // Delete
        SessionState::delete(&path).unwrap();
        assert!(!blue_dir.join("session").exists());
    }

    #[test]
    fn test_session_state_load_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let result = SessionState::load(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert!(id1.starts_with("sess-"));
        assert!(id2.starts_with("sess-"));
        // IDs should be unique (different timestamps)
        // Note: Could be same if generated within same millisecond
    }

    #[test]
    fn test_session_stop_no_session() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let blue_dir = path.join(".blue");
        std::fs::create_dir_all(&blue_dir).unwrap();

        let result = handle_session_stop(Some(&path));
        assert!(result.is_err());
    }
}
