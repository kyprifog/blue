//! Git repository detection and operations for Blue
//!
//! Finds Blue's home (.blue/) and manages worktrees.

use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

/// Blue's directory structure detection result
#[derive(Debug, Clone)]
pub struct BlueHome {
    /// Root directory containing .blue/
    pub root: PathBuf,
    /// Path to .blue/repos/ (markdown docs)
    pub repos_path: PathBuf,
    /// Path to .blue/data/ (SQLite databases)
    pub data_path: PathBuf,
    /// Path to .blue/worktrees/ (git worktrees)
    pub worktrees_path: PathBuf,
    /// Detected project name
    pub project_name: Option<String>,
}

impl BlueHome {
    /// Get the docs path for a specific project
    pub fn docs_path(&self, project: &str) -> PathBuf {
        self.repos_path.join(project).join("docs")
    }

    /// Get the database path for a specific project
    pub fn db_path(&self, project: &str) -> PathBuf {
        self.data_path.join(project).join("blue.db")
    }
}

/// Information about a git worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Path to the worktree
    pub path: PathBuf,
    /// Branch name
    pub branch: String,
    /// Whether this is the main worktree
    pub is_main: bool,
}

impl WorktreeInfo {
    /// Extract RFC title from branch name if it follows the pattern rfc/{title}
    pub fn rfc_title(&self) -> Option<String> {
        if self.branch.starts_with("rfc/") {
            Some(self.branch.trim_start_matches("rfc/").to_string())
        } else {
            None
        }
    }
}

/// Repository errors
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Can't find Blue here. Run 'blue init' first?")]
    NotHome,

    #[error("Git trouble: {0}")]
    Git(#[from] git2::Error),

    #[error("Can't read directory: {0}")]
    Io(#[from] std::io::Error),
}

/// Detect Blue's home directory structure
///
/// Looks for .blue/ in the current directory or any parent.
/// The structure is:
/// ```text
/// project/
/// ├── .blue/
/// │   ├── repos/         # Cloned repos with docs/
/// │   ├── data/          # SQLite databases
/// │   └── worktrees/     # Git worktrees
/// └── ...
/// ```
pub fn detect_blue(from: &Path) -> Result<BlueHome, RepoError> {
    let mut current = from.to_path_buf();

    loop {
        let blue_dir = current.join(".blue");
        if blue_dir.exists() && blue_dir.is_dir() {
            debug!("Found Blue's home at {:?}", blue_dir);

            return Ok(BlueHome {
                root: current.clone(),
                repos_path: blue_dir.join("repos"),
                data_path: blue_dir.join("data"),
                worktrees_path: blue_dir.join("worktrees"),
                project_name: extract_project_name(&current),
            });
        }

        // Also check for legacy .repos/.data/.worktrees structure
        let legacy_repos = current.join(".repos");
        let legacy_data = current.join(".data");
        if legacy_repos.exists() && legacy_data.exists() {
            debug!("Found legacy Blue structure at {:?}", current);

            return Ok(BlueHome {
                root: current.clone(),
                repos_path: legacy_repos,
                data_path: legacy_data,
                worktrees_path: current.join(".worktrees"),
                project_name: extract_project_name(&current),
            });
        }

        if !current.pop() {
            break;
        }
    }

    Err(RepoError::NotHome)
}

/// Extract project name from git remote or directory name
fn extract_project_name(path: &Path) -> Option<String> {
    // Try git remote first
    if let Ok(repo) = git2::Repository::discover(path) {
        if let Ok(remote) = repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return extract_repo_name_from_url(url);
            }
        }
    }

    // Fall back to directory name
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Extract repository name from a git URL
fn extract_repo_name_from_url(url: &str) -> Option<String> {
    // Handle SSH URLs: git@host:org/repo.git
    if url.contains(':') && !url.contains("://") {
        let after_colon = url.split(':').last()?;
        let name = after_colon.trim_end_matches(".git");
        return name.split('/').last().map(|s| s.to_string());
    }

    // Handle HTTPS URLs: https://host/org/repo.git
    let name = url.trim_end_matches(".git");
    name.split('/').last().map(|s| s.to_string())
}

/// List git worktrees for a repository
pub fn list_worktrees(repo: &git2::Repository) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();

    // Add main worktree
    if let Some(workdir) = repo.workdir() {
        if let Ok(head) = repo.head() {
            let branch = head
                .shorthand()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "HEAD".to_string());

            worktrees.push(WorktreeInfo {
                path: workdir.to_path_buf(),
                branch,
                is_main: true,
            });
        }
    }

    // Add other worktrees
    if let Ok(wt_names) = repo.worktrees() {
        for name in wt_names.iter().flatten() {
            if let Ok(wt) = repo.find_worktree(name) {
                if let Some(path) = wt.path().to_str() {
                    // Try to get the branch for this worktree
                    let branch = wt.name().unwrap_or("unknown").to_string();

                    worktrees.push(WorktreeInfo {
                        path: PathBuf::from(path),
                        branch,
                        is_main: false,
                    });
                }
            }
        }
    }

    worktrees
}

/// Create a new worktree for an RFC
pub fn create_worktree(
    repo: &git2::Repository,
    branch_name: &str,
    worktree_path: &Path,
) -> Result<(), RepoError> {
    // Create the branch if it doesn't exist
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(branch) => branch,
        Err(_) => repo.branch(branch_name, &head_commit, false)?,
    };

    // Create the worktree
    let reference = branch.into_reference();
    repo.worktree(
        branch_name,
        worktree_path,
        Some(git2::WorktreeAddOptions::new().reference(Some(&reference))),
    )?;

    Ok(())
}

/// Remove a worktree
pub fn remove_worktree(repo: &git2::Repository, name: &str) -> Result<(), RepoError> {
    let worktree = repo.find_worktree(name)?;

    // Prune the worktree (this removes the worktree but keeps the branch)
    worktree.prune(Some(
        git2::WorktreePruneOptions::new()
            .valid(true)
            .working_tree(true),
    ))?;

    Ok(())
}

/// Check if a branch is merged into another
pub fn is_branch_merged(
    repo: &git2::Repository,
    branch: &str,
    into: &str,
) -> Result<bool, RepoError> {
    let branch_commit = repo
        .find_branch(branch, git2::BranchType::Local)?
        .get()
        .peel_to_commit()?
        .id();

    let into_commit = repo
        .find_branch(into, git2::BranchType::Local)?
        .get()
        .peel_to_commit()?
        .id();

    // Check if branch_commit is an ancestor of into_commit
    Ok(repo.graph_descendant_of(into_commit, branch_commit)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name_ssh() {
        let url = "git@github.com:superviber/blue.git";
        assert_eq!(extract_repo_name_from_url(url), Some("blue".to_string()));
    }

    #[test]
    fn test_extract_repo_name_https() {
        let url = "https://github.com/superviber/blue.git";
        assert_eq!(extract_repo_name_from_url(url), Some("blue".to_string()));
    }

    #[test]
    fn test_worktree_info_rfc_title() {
        let wt = WorktreeInfo {
            path: PathBuf::from("/tmp/test"),
            branch: "rfc/my-feature".to_string(),
            is_main: false,
        };
        assert_eq!(wt.rfc_title(), Some("my-feature".to_string()));

        let main = WorktreeInfo {
            path: PathBuf::from("/tmp/main"),
            branch: "main".to_string(),
            is_main: true,
        };
        assert_eq!(main.rfc_title(), None);
    }
}
