//! Project state management for Blue
//!
//! Aggregates documents, worktrees, and provides status views.

use std::collections::HashSet;

use serde::Serialize;
use tracing::warn;

use crate::repo::{list_worktrees, BlueHome, WorktreeInfo};
use crate::store::{DocType, DocumentStore, StoreError};

/// Complete project state
#[derive(Debug)]
pub struct ProjectState {
    /// Blue's home directory
    pub home: BlueHome,
    /// SQLite document store
    pub store: DocumentStore,
    /// Active git worktrees
    pub worktrees: Vec<WorktreeInfo>,
    /// Set of RFC titles with active worktrees
    worktree_rfcs: HashSet<String>,
    /// Project name
    pub project: String,
}

impl ProjectState {
    /// Load project state
    pub fn load(home: BlueHome, project: &str) -> Result<Self, StateError> {
        let db_path = home.db_path(project);
        let store = DocumentStore::open(&db_path)?;

        // Discover worktrees
        let worktrees = Self::discover_worktrees(&home, project);
        let worktree_rfcs: HashSet<String> =
            worktrees.iter().filter_map(|wt| wt.rfc_title()).collect();

        Ok(Self {
            home,
            store,
            worktrees,
            worktree_rfcs,
            project: project.to_string(),
        })
    }

    /// Discover worktrees from the repo
    fn discover_worktrees(home: &BlueHome, project: &str) -> Vec<WorktreeInfo> {
        let repo_path = home.repos_path.join(project);

        if let Ok(repo) = git2::Repository::open(&repo_path) {
            return list_worktrees(&repo);
        }

        // Also try from root
        if let Ok(repo) = git2::Repository::discover(&home.root) {
            return list_worktrees(&repo);
        }

        Vec::new()
    }

    /// Reload state from disk
    pub fn reload(&mut self) -> Result<(), StateError> {
        self.worktrees = Self::discover_worktrees(&self.home, &self.project);
        self.worktree_rfcs = self
            .worktrees
            .iter()
            .filter_map(|wt| wt.rfc_title())
            .collect();
        Ok(())
    }

    /// Get RFCs that are in-progress with active worktrees
    pub fn active_items(&self) -> Vec<WorkItem> {
        match self.store.list_documents_by_status(DocType::Rfc, "in-progress") {
            Ok(docs) => docs
                .into_iter()
                .filter(|doc| self.worktree_rfcs.contains(&doc.title))
                .map(|doc| WorkItem {
                    title: doc.title,
                    status: doc.status,
                    has_worktree: true,
                    item_type: ItemType::Rfc,
                })
                .collect(),
            Err(e) => {
                warn!("Couldn't get active items: {}", e);
                Vec::new()
            }
        }
    }

    /// Get RFCs that are accepted and ready to start
    pub fn ready_items(&self) -> Vec<WorkItem> {
        match self.store.list_documents_by_status(DocType::Rfc, "accepted") {
            Ok(docs) => docs
                .into_iter()
                .map(|doc| WorkItem {
                    title: doc.title,
                    status: doc.status,
                    has_worktree: false,
                    item_type: ItemType::Rfc,
                })
                .collect(),
            Err(e) => {
                warn!("Couldn't get ready items: {}", e);
                Vec::new()
            }
        }
    }

    /// Get RFCs that are in-progress but have no worktree (possibly stalled)
    pub fn stalled_items(&self) -> Vec<WorkItem> {
        match self.store.list_documents_by_status(DocType::Rfc, "in-progress") {
            Ok(docs) => docs
                .into_iter()
                .filter(|doc| !self.worktree_rfcs.contains(&doc.title))
                .map(|doc| WorkItem {
                    title: doc.title,
                    status: doc.status,
                    has_worktree: false,
                    item_type: ItemType::Rfc,
                })
                .collect(),
            Err(e) => {
                warn!("Couldn't get stalled items: {}", e);
                Vec::new()
            }
        }
    }

    /// Get draft RFCs
    pub fn draft_items(&self) -> Vec<WorkItem> {
        match self.store.list_documents_by_status(DocType::Rfc, "draft") {
            Ok(docs) => docs
                .into_iter()
                .map(|doc| WorkItem {
                    title: doc.title,
                    status: doc.status,
                    has_worktree: false,
                    item_type: ItemType::Rfc,
                })
                .collect(),
            Err(e) => {
                warn!("Couldn't get draft items: {}", e);
                Vec::new()
            }
        }
    }

    /// Check if an RFC has an active worktree
    pub fn has_worktree(&self, rfc_title: &str) -> bool {
        self.worktree_rfcs.contains(rfc_title)
    }

    /// Generate a status hint for the user
    pub fn generate_hint(&self) -> String {
        let active = self.active_items();
        let ready = self.ready_items();
        let stalled = self.stalled_items();

        if !stalled.is_empty() {
            return format!(
                "'{}' might be stalled - it's in-progress but has no worktree",
                stalled[0].title
            );
        }

        if !ready.is_empty() {
            return format!("'{}' is ready to implement. Want to start?", ready[0].title);
        }

        if !active.is_empty() {
            return format!("{} item(s) currently in progress", active.len());
        }

        "Nothing pressing right now. Good time to plan?".to_string()
    }

    /// Get project status summary
    pub fn status_summary(&self) -> StatusSummary {
        let active = self.active_items();
        let ready = self.ready_items();
        let stalled = self.stalled_items();
        let drafts = self.draft_items();

        StatusSummary {
            active_count: active.len(),
            ready_count: ready.len(),
            stalled_count: stalled.len(),
            draft_count: drafts.len(),
            active,
            ready,
            stalled,
            drafts,
            hint: self.generate_hint(),
        }
    }
}

/// A work item (RFC, spike, etc.)
#[derive(Debug, Clone, Serialize)]
pub struct WorkItem {
    pub title: String,
    pub status: String,
    pub has_worktree: bool,
    pub item_type: ItemType,
}

/// Type of work item
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Rfc,
    Spike,
    Adr,
    Decision,
    Prd,
}

/// Project status summary
#[derive(Debug, Clone, Serialize)]
pub struct StatusSummary {
    pub active_count: usize,
    pub ready_count: usize,
    pub stalled_count: usize,
    pub draft_count: usize,
    pub active: Vec<WorkItem>,
    pub ready: Vec<WorkItem>,
    pub stalled: Vec<WorkItem>,
    pub drafts: Vec<WorkItem>,
    pub hint: String,
}

/// State errors
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Store error: {0}")]
    Store(#[from] StoreError),

    #[error("Repo error: {0}")]
    Repo(#[from] crate::repo::RepoError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hint_empty() {
        // This would require setting up a full test environment
        // For now, just verify the function exists and compiles
    }
}
