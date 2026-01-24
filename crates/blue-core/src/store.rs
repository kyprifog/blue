//! SQLite document store for Blue
//!
//! Persistence layer for RFCs, Spikes, ADRs, and other documents.

use std::path::Path;
use std::thread;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use tracing::{debug, info, warn};

/// Current schema version
const SCHEMA_VERSION: i32 = 1;

/// Core database schema
const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS documents (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        doc_type TEXT NOT NULL,
        number INTEGER,
        title TEXT NOT NULL,
        status TEXT NOT NULL,
        file_path TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        UNIQUE(doc_type, title)
    );

    CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(doc_type);
    CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(doc_type, status);

    CREATE TABLE IF NOT EXISTS document_links (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_id INTEGER NOT NULL,
        target_id INTEGER NOT NULL,
        link_type TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (source_id) REFERENCES documents(id) ON DELETE CASCADE,
        FOREIGN KEY (target_id) REFERENCES documents(id) ON DELETE CASCADE,
        UNIQUE(source_id, target_id, link_type)
    );

    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id INTEGER NOT NULL,
        task_index INTEGER NOT NULL,
        description TEXT NOT NULL,
        completed INTEGER NOT NULL DEFAULT 0,
        completed_at TEXT,
        FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
        UNIQUE(document_id, task_index)
    );

    CREATE TABLE IF NOT EXISTS worktrees (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id INTEGER NOT NULL,
        branch_name TEXT NOT NULL,
        worktree_path TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
        UNIQUE(document_id)
    );

    CREATE TABLE IF NOT EXISTS metadata (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id INTEGER NOT NULL,
        key TEXT NOT NULL,
        value TEXT NOT NULL,
        FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
        UNIQUE(document_id, key)
    );
"#;

/// FTS5 schema for full-text search
const FTS5_SCHEMA: &str = r#"
    CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
        title,
        content,
        doc_type,
        content=documents,
        content_rowid=id
    );

    CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
        INSERT INTO documents_fts(rowid, title, doc_type)
        VALUES (new.id, new.title, new.doc_type);
    END;

    CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
        INSERT INTO documents_fts(documents_fts, rowid, title, doc_type)
        VALUES ('delete', old.id, old.title, old.doc_type);
    END;

    CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
        INSERT INTO documents_fts(documents_fts, rowid, title, doc_type)
        VALUES ('delete', old.id, old.title, old.doc_type);
        INSERT INTO documents_fts(rowid, title, doc_type)
        VALUES (new.id, new.title, new.doc_type);
    END;
"#;

/// Document types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocType {
    Rfc,
    Spike,
    Adr,
    Decision,
    Prd,
}

impl DocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocType::Rfc => "rfc",
            DocType::Spike => "spike",
            DocType::Adr => "adr",
            DocType::Decision => "decision",
            DocType::Prd => "prd",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rfc" => Some(DocType::Rfc),
            "spike" => Some(DocType::Spike),
            "adr" => Some(DocType::Adr),
            "decision" => Some(DocType::Decision),
            "prd" => Some(DocType::Prd),
            _ => None,
        }
    }

    /// Human-readable plural for Blue's messages
    pub fn plural(&self) -> &'static str {
        match self {
            DocType::Rfc => "RFCs",
            DocType::Spike => "spikes",
            DocType::Adr => "ADRs",
            DocType::Decision => "decisions",
            DocType::Prd => "PRDs",
        }
    }
}

/// Link types between documents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Spike leads to RFC
    SpikeToRfc,
    /// RFC leads to ADR
    RfcToAdr,
    /// PRD leads to RFC
    PrdToRfc,
    /// Generic reference
    References,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkType::SpikeToRfc => "spike_to_rfc",
            LinkType::RfcToAdr => "rfc_to_adr",
            LinkType::PrdToRfc => "prd_to_rfc",
            LinkType::References => "references",
        }
    }
}

/// A document in the store
#[derive(Debug, Clone)]
pub struct Document {
    pub id: Option<i64>,
    pub doc_type: DocType,
    pub number: Option<i32>,
    pub title: String,
    pub status: String,
    pub file_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Document {
    /// Create a new document
    pub fn new(doc_type: DocType, title: &str, status: &str) -> Self {
        Self {
            id: None,
            doc_type,
            number: None,
            title: title.to_string(),
            status: status.to_string(),
            file_path: None,
            created_at: None,
            updated_at: None,
        }
    }
}

/// A task in a document's plan
#[derive(Debug, Clone)]
pub struct Task {
    pub id: Option<i64>,
    pub document_id: i64,
    pub task_index: i32,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
}

/// Task completion progress
#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub completed: usize,
    pub total: usize,
    pub percentage: usize,
}

/// A worktree associated with a document
#[derive(Debug, Clone)]
pub struct Worktree {
    pub id: Option<i64>,
    pub document_id: i64,
    pub branch_name: String,
    pub worktree_path: String,
    pub created_at: Option<String>,
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document: Document,
    pub score: f64,
    pub snippet: Option<String>,
}

/// Store errors - in Blue's voice
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Can't find '{0}'. Check the name's spelled right?")]
    NotFound(String),

    #[error("Database hiccup: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("'{0}' already exists. Want to update it instead?")]
    AlreadyExists(String),

    #[error("Can't do that: {0}")]
    InvalidOperation(String),
}

/// Check if an error is a busy/locked error
fn is_busy_error(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::DatabaseBusy,
                ..
            },
            _
        ) | rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::DatabaseLocked,
                ..
            },
            _
        )
    )
}

/// SQLite-based document store
pub struct DocumentStore {
    conn: Connection,
}

impl std::fmt::Debug for DocumentStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentStore")
            .field("conn", &"<Connection>")
            .finish()
    }
}

impl DocumentStore {
    /// Open or create a document store
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        info!("Opening Blue's document store at {:?}", path);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(path)?;

        // Configure for concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Get a reference to the underlying connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), StoreError> {
        let version: Option<i32> = self
            .conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                row.get(0)
            })
            .ok();

        match version {
            None => {
                debug!("Setting up Blue's database (version {})", SCHEMA_VERSION);
                self.conn.execute_batch(SCHEMA)?;
                self.conn.execute_batch(FTS5_SCHEMA)?;
                self.conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    params![SCHEMA_VERSION],
                )?;
            }
            Some(v) if v == SCHEMA_VERSION => {
                debug!("Database is up to date (version {})", v);
            }
            Some(v) => {
                warn!(
                    "Schema version {} found, expected {}. Migrations may be needed.",
                    v, SCHEMA_VERSION
                );
            }
        }

        Ok(())
    }

    /// Execute with retry on busy
    fn with_retry<F, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: Fn() -> Result<T, StoreError>,
    {
        let mut attempts = 0;
        loop {
            match f() {
                Ok(result) => return Ok(result),
                Err(StoreError::Database(ref e)) if is_busy_error(e) && attempts < 3 => {
                    attempts += 1;
                    let delay = Duration::from_millis(100 * attempts as u64);
                    debug!("Database busy, retrying in {:?}", delay);
                    thread::sleep(delay);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Begin a write transaction
    pub fn begin_write(&mut self) -> Result<Transaction<'_>, StoreError> {
        Ok(self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?)
    }

    // ==================== Document Operations ====================

    /// Add a new document
    pub fn add_document(&self, doc: &Document) -> Result<i64, StoreError> {
        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO documents (doc_type, number, title, status, file_path, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    doc.doc_type.as_str(),
                    doc.number,
                    doc.title,
                    doc.status,
                    doc.file_path,
                    now,
                    now,
                ],
            )?;
            Ok(self.conn.last_insert_rowid())
        })
    }

    /// Get a document by type and title
    pub fn get_document(&self, doc_type: DocType, title: &str) -> Result<Document, StoreError> {
        self.conn
            .query_row(
                "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
                 FROM documents WHERE doc_type = ?1 AND title = ?2",
                params![doc_type.as_str(), title],
                |row| {
                    Ok(Document {
                        id: Some(row.get(0)?),
                        doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                        number: row.get(2)?,
                        title: row.get(3)?,
                        status: row.get(4)?,
                        file_path: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(title.to_string()),
                e => StoreError::Database(e),
            })
    }

    /// Get a document by ID
    pub fn get_document_by_id(&self, id: i64) -> Result<Document, StoreError> {
        self.conn
            .query_row(
                "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
                 FROM documents WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Document {
                        id: Some(row.get(0)?),
                        doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                        number: row.get(2)?,
                        title: row.get(3)?,
                        status: row.get(4)?,
                        file_path: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StoreError::NotFound(format!("document #{}", id))
                }
                e => StoreError::Database(e),
            })
    }

    /// Get a document by number
    pub fn get_document_by_number(
        &self,
        doc_type: DocType,
        number: i32,
    ) -> Result<Document, StoreError> {
        self.conn
            .query_row(
                "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
                 FROM documents WHERE doc_type = ?1 AND number = ?2",
                params![doc_type.as_str(), number],
                |row| {
                    Ok(Document {
                        id: Some(row.get(0)?),
                        doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                        number: row.get(2)?,
                        title: row.get(3)?,
                        status: row.get(4)?,
                        file_path: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StoreError::NotFound(format!("{} #{}", doc_type.as_str(), number))
                }
                e => StoreError::Database(e),
            })
    }

    /// Find a document using flexible matching
    pub fn find_document(&self, doc_type: DocType, query: &str) -> Result<Document, StoreError> {
        // Try exact match first
        if let Ok(doc) = self.get_document(doc_type, query) {
            return Ok(doc);
        }

        // Try number match
        let trimmed = query.trim_start_matches('0');
        if let Ok(num) = if trimmed.is_empty() {
            "0".parse()
        } else {
            trimmed.parse::<i32>()
        } {
            if let Ok(doc) = self.get_document_by_number(doc_type, num) {
                return Ok(doc);
            }
        }

        // Try substring match
        let pattern = format!("%{}%", query.to_lowercase());
        if let Ok(doc) = self.conn.query_row(
            "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
             FROM documents WHERE doc_type = ?1 AND LOWER(title) LIKE ?2
             ORDER BY LENGTH(title) ASC LIMIT 1",
            params![doc_type.as_str(), pattern],
            |row| {
                Ok(Document {
                    id: Some(row.get(0)?),
                    doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                    number: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    file_path: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        ) {
            return Ok(doc);
        }

        Err(StoreError::NotFound(format!(
            "{} matching '{}'",
            doc_type.as_str(),
            query
        )))
    }

    /// Update a document's status
    pub fn update_document_status(
        &self,
        doc_type: DocType,
        title: &str,
        status: &str,
    ) -> Result<(), StoreError> {
        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            let updated = self.conn.execute(
                "UPDATE documents SET status = ?1, updated_at = ?2 WHERE doc_type = ?3 AND title = ?4",
                params![status, now, doc_type.as_str(), title],
            )?;
            if updated == 0 {
                return Err(StoreError::NotFound(title.to_string()));
            }
            Ok(())
        })
    }

    /// Update a document
    pub fn update_document(&self, doc: &Document) -> Result<(), StoreError> {
        let id = doc
            .id
            .ok_or_else(|| StoreError::InvalidOperation("Document has no ID".to_string()))?;

        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            let updated = self.conn.execute(
                "UPDATE documents SET doc_type = ?1, number = ?2, title = ?3, status = ?4,
                 file_path = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    doc.doc_type.as_str(),
                    doc.number,
                    doc.title,
                    doc.status,
                    doc.file_path,
                    now,
                    id
                ],
            )?;
            if updated == 0 {
                return Err(StoreError::NotFound(format!("document #{}", id)));
            }
            Ok(())
        })
    }

    /// List all documents of a given type
    pub fn list_documents(&self, doc_type: DocType) -> Result<Vec<Document>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
             FROM documents WHERE doc_type = ?1 ORDER BY number DESC, title ASC",
        )?;

        let rows = stmt.query_map(params![doc_type.as_str()], |row| {
            Ok(Document {
                id: Some(row.get(0)?),
                doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                number: row.get(2)?,
                title: row.get(3)?,
                status: row.get(4)?,
                file_path: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }

    /// List documents by status
    pub fn list_documents_by_status(
        &self,
        doc_type: DocType,
        status: &str,
    ) -> Result<Vec<Document>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, doc_type, number, title, status, file_path, created_at, updated_at
             FROM documents WHERE doc_type = ?1 AND status = ?2 ORDER BY number DESC, title ASC",
        )?;

        let rows = stmt.query_map(params![doc_type.as_str(), status], |row| {
            Ok(Document {
                id: Some(row.get(0)?),
                doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                number: row.get(2)?,
                title: row.get(3)?,
                status: row.get(4)?,
                file_path: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }

    /// Delete a document
    pub fn delete_document(&self, doc_type: DocType, title: &str) -> Result<(), StoreError> {
        self.with_retry(|| {
            let deleted = self.conn.execute(
                "DELETE FROM documents WHERE doc_type = ?1 AND title = ?2",
                params![doc_type.as_str(), title],
            )?;
            if deleted == 0 {
                return Err(StoreError::NotFound(title.to_string()));
            }
            Ok(())
        })
    }

    /// Get the next document number for a type
    pub fn next_number(&self, doc_type: DocType) -> Result<i32, StoreError> {
        let max: Option<i32> = self.conn.query_row(
            "SELECT MAX(number) FROM documents WHERE doc_type = ?1",
            params![doc_type.as_str()],
            |row| row.get(0),
        )?;
        Ok(max.unwrap_or(0) + 1)
    }

    // ==================== Link Operations ====================

    /// Link two documents
    pub fn link_documents(
        &self,
        source_id: i64,
        target_id: i64,
        link_type: LinkType,
    ) -> Result<(), StoreError> {
        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT OR IGNORE INTO document_links (source_id, target_id, link_type, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![source_id, target_id, link_type.as_str(), now],
            )?;
            Ok(())
        })
    }

    /// Get linked documents
    pub fn get_linked_documents(
        &self,
        source_id: i64,
        link_type: Option<LinkType>,
    ) -> Result<Vec<Document>, StoreError> {
        let query = match link_type {
            Some(lt) => format!(
                "SELECT d.id, d.doc_type, d.number, d.title, d.status, d.file_path, d.created_at, d.updated_at
                 FROM documents d
                 JOIN document_links l ON l.target_id = d.id
                 WHERE l.source_id = ?1 AND l.link_type = '{}'",
                lt.as_str()
            ),
            None => "SELECT d.id, d.doc_type, d.number, d.title, d.status, d.file_path, d.created_at, d.updated_at
                     FROM documents d
                     JOIN document_links l ON l.target_id = d.id
                     WHERE l.source_id = ?1".to_string(),
        };

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params![source_id], |row| {
            Ok(Document {
                id: Some(row.get(0)?),
                doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                number: row.get(2)?,
                title: row.get(3)?,
                status: row.get(4)?,
                file_path: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }

    // ==================== Task Operations ====================

    /// Set tasks for a document (replaces existing)
    pub fn set_tasks(&self, document_id: i64, tasks: &[String]) -> Result<(), StoreError> {
        self.with_retry(|| {
            self.conn
                .execute("DELETE FROM tasks WHERE document_id = ?1", params![document_id])?;

            for (idx, desc) in tasks.iter().enumerate() {
                self.conn.execute(
                    "INSERT INTO tasks (document_id, task_index, description, completed)
                     VALUES (?1, ?2, ?3, 0)",
                    params![document_id, (idx + 1) as i32, desc],
                )?;
            }

            Ok(())
        })
    }

    /// Mark a task as complete
    pub fn complete_task(&self, document_id: i64, task_index: i32) -> Result<(), StoreError> {
        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            let updated = self.conn.execute(
                "UPDATE tasks SET completed = 1, completed_at = ?1
                 WHERE document_id = ?2 AND task_index = ?3",
                params![now, document_id, task_index],
            )?;
            if updated == 0 {
                return Err(StoreError::NotFound(format!(
                    "task {} in document #{}",
                    task_index, document_id
                )));
            }
            Ok(())
        })
    }

    /// Get task progress
    pub fn get_task_progress(&self, document_id: i64) -> Result<TaskProgress, StoreError> {
        let (total, completed): (i64, i64) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(completed), 0) FROM tasks WHERE document_id = ?1",
            params![document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let total = total as usize;
        let completed = completed as usize;
        let percentage = if total > 0 {
            (completed * 100) / total
        } else {
            0
        };

        Ok(TaskProgress {
            completed,
            total,
            percentage,
        })
    }

    /// Get all tasks for a document
    pub fn get_tasks(&self, document_id: i64) -> Result<Vec<Task>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, task_index, description, completed, completed_at
             FROM tasks WHERE document_id = ?1 ORDER BY task_index",
        )?;

        let rows = stmt.query_map(params![document_id], |row| {
            Ok(Task {
                id: Some(row.get(0)?),
                document_id: row.get(1)?,
                task_index: row.get(2)?,
                description: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                completed_at: row.get(5)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }

    // ==================== Worktree Operations ====================

    /// Add a worktree for a document
    pub fn add_worktree(&self, worktree: &Worktree) -> Result<i64, StoreError> {
        self.with_retry(|| {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO worktrees (document_id, branch_name, worktree_path, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    worktree.document_id,
                    worktree.branch_name,
                    worktree.worktree_path,
                    now
                ],
            )?;
            Ok(self.conn.last_insert_rowid())
        })
    }

    /// Get worktree for a document
    pub fn get_worktree(&self, document_id: i64) -> Result<Option<Worktree>, StoreError> {
        self.conn
            .query_row(
                "SELECT id, document_id, branch_name, worktree_path, created_at
                 FROM worktrees WHERE document_id = ?1",
                params![document_id],
                |row| {
                    Ok(Worktree {
                        id: Some(row.get(0)?),
                        document_id: row.get(1)?,
                        branch_name: row.get(2)?,
                        worktree_path: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(StoreError::Database)
    }

    /// Remove a worktree
    pub fn remove_worktree(&self, document_id: i64) -> Result<(), StoreError> {
        self.with_retry(|| {
            self.conn.execute(
                "DELETE FROM worktrees WHERE document_id = ?1",
                params![document_id],
            )?;
            Ok(())
        })
    }

    /// List all worktrees
    pub fn list_worktrees(&self) -> Result<Vec<Worktree>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, branch_name, worktree_path, created_at FROM worktrees",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Worktree {
                id: Some(row.get(0)?),
                document_id: row.get(1)?,
                branch_name: row.get(2)?,
                worktree_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }

    // ==================== Search Operations ====================

    /// Search documents using FTS5
    pub fn search_documents(
        &self,
        query: &str,
        doc_type: Option<DocType>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StoreError> {
        let escaped = query.replace('"', "\"\"");
        let fts_query = format!("\"{}\"*", escaped);

        let sql = match doc_type {
            Some(dt) => format!(
                "SELECT d.id, d.doc_type, d.number, d.title, d.status, d.file_path,
                        d.created_at, d.updated_at, bm25(documents_fts) as score
                 FROM documents_fts fts
                 JOIN documents d ON d.id = fts.rowid
                 WHERE documents_fts MATCH ?1 AND d.doc_type = '{}'
                 ORDER BY score
                 LIMIT ?2",
                dt.as_str()
            ),
            None => "SELECT d.id, d.doc_type, d.number, d.title, d.status, d.file_path,
                            d.created_at, d.updated_at, bm25(documents_fts) as score
                     FROM documents_fts fts
                     JOIN documents d ON d.id = fts.rowid
                     WHERE documents_fts MATCH ?1
                     ORDER BY score
                     LIMIT ?2"
                .to_string(),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fts_query, limit as i32], |row| {
            Ok(SearchResult {
                document: Document {
                    id: Some(row.get(0)?),
                    doc_type: DocType::from_str(row.get::<_, String>(1)?.as_str()).unwrap(),
                    number: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    file_path: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                },
                score: row.get(8)?,
                snippet: None,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_find_document() {
        let store = DocumentStore::open_in_memory().unwrap();

        let doc = Document::new(DocType::Rfc, "test-feature", "draft");
        let id = store.add_document(&doc).unwrap();

        let found = store.find_document(DocType::Rfc, "test-feature").unwrap();
        assert_eq!(found.id, Some(id));
        assert_eq!(found.title, "test-feature");
    }

    #[test]
    fn test_task_progress() {
        let store = DocumentStore::open_in_memory().unwrap();

        let doc = Document::new(DocType::Rfc, "task-test", "draft");
        let id = store.add_document(&doc).unwrap();

        store
            .set_tasks(id, &["Task 1".into(), "Task 2".into(), "Task 3".into()])
            .unwrap();

        let progress = store.get_task_progress(id).unwrap();
        assert_eq!(progress.total, 3);
        assert_eq!(progress.completed, 0);
        assert_eq!(progress.percentage, 0);

        store.complete_task(id, 1).unwrap();

        let progress = store.get_task_progress(id).unwrap();
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.percentage, 33);
    }
}
