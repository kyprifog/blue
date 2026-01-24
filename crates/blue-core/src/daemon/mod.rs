//! Blue Daemon - Per-machine service for realm coordination
//!
//! The daemon manages:
//! - Realm state and git operations
//! - Session tracking
//! - Notifications between repos
//!
//! Architecture:
//! - HTTP server on localhost:7865
//! - SQLite database at ~/.blue/daemon.db
//! - Realm repos cloned to ~/.blue/realms/

mod client;
mod db;
mod paths;
mod server;

pub use client::{ClientError, CreateSessionRequest, DaemonClient, HealthResponse, SyncResponse};
pub use db::{DaemonDb, DaemonDbError, Notification, Realm, RealmStatus, Session};
pub use paths::{DaemonPaths, PathError};
pub use server::{run_daemon, DaemonState};

/// Default port for the daemon HTTP server
pub const DAEMON_PORT: u16 = 7865;

/// Daemon version for API compatibility checks
pub const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");
