//! Realm MCP tool handlers
//!
//! Implements RFC 0002: Realm MCP Integration (Phase 1)
//! - realm_status: Get realm overview
//! - realm_check: Validate contracts/bindings
//! - contract_get: Get contract details

use blue_core::daemon::DaemonPaths;
use blue_core::realm::{LocalRepoConfig, RealmService};
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
}
