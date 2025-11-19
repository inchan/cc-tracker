//! Database migration system
//!
//! Manages database schema versions and migrations.

use rusqlite::{params, Connection};

use crate::{PromptTrackingError, Result};

/// Current schema version
pub const CURRENT_VERSION: i32 = 2;

/// Migration manager for database schema updates
pub struct MigrationManager<'a> {
    conn: &'a Connection,
}

impl<'a> MigrationManager<'a> {
    /// Create a new migration manager
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Run all pending migrations
    pub fn run_migrations(&self) -> Result<()> {
        self.ensure_schema_version_table()?;
        let current_version = self.get_current_version()?;

        if current_version < CURRENT_VERSION {
            eprintln!(
                "Migrating database from version {} to {}...",
                current_version, CURRENT_VERSION
            );

            for version in (current_version + 1)..=CURRENT_VERSION {
                self.run_migration(version)?;
                self.set_version(version)?;
                eprintln!("Applied migration v{}", version);
            }

            eprintln!("Database migration complete.");
        }

        Ok(())
    }

    /// Ensure the schema_version table exists
    fn ensure_schema_version_table(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_version (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                )",
                [],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!(
                    "Failed to create schema_version table: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Get current schema version
    fn get_current_version(&self) -> Result<i32> {
        let version: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get schema version: {}", e))
            })?;

        Ok(version)
    }

    /// Set current schema version
    fn set_version(&self, version: i32) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
                params![version],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to set schema version: {}", e))
            })?;

        Ok(())
    }

    /// Run a specific migration
    fn run_migration(&self, version: i32) -> Result<()> {
        match version {
            1 => self.migration_v1(),
            2 => self.migration_v2(),
            _ => Err(PromptTrackingError::DatabaseError(format!(
                "Unknown migration version: {}",
                version
            ))),
        }
    }

    /// Migration v1: Initial schema (already applied if database exists)
    fn migration_v1(&self) -> Result<()> {
        // This is the initial schema - skip if tables already exist
        let tables_exist: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='prompts'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if tables_exist == 0 {
            self.conn
                .execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS prompts (
                        id TEXT PRIMARY KEY,
                        content TEXT NOT NULL,
                        content_hash TEXT NOT NULL,
                        category TEXT,
                        status TEXT NOT NULL DEFAULT 'active',
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        model TEXT NOT NULL,
                        input_tokens INTEGER,
                        output_tokens INTEGER,
                        execution_time_ms INTEGER,
                        estimated_cost REAL,
                        context TEXT
                    );

                    CREATE TABLE IF NOT EXISTS tags (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT UNIQUE NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS prompt_tags (
                        prompt_id TEXT NOT NULL,
                        tag_id INTEGER NOT NULL,
                        PRIMARY KEY (prompt_id, tag_id),
                        FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
                        FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                    );

                    CREATE TABLE IF NOT EXISTS quality_scores (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        prompt_id TEXT NOT NULL,
                        total_score REAL NOT NULL,
                        clarity REAL NOT NULL,
                        completeness REAL NOT NULL,
                        specificity REAL NOT NULL,
                        guidance REAL NOT NULL,
                        analyzed_at TEXT NOT NULL,
                        FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
                    );

                    CREATE TABLE IF NOT EXISTS efficiency_metrics (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        prompt_id TEXT NOT NULL,
                        efficiency_score REAL NOT NULL,
                        token_efficiency REAL NOT NULL,
                        time_efficiency REAL NOT NULL,
                        cost_efficiency REAL NOT NULL,
                        calculated_at TEXT NOT NULL,
                        FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
                    );

                    CREATE TABLE IF NOT EXISTS version_history (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        prompt_id TEXT NOT NULL,
                        content TEXT NOT NULL,
                        content_hash TEXT NOT NULL,
                        version INTEGER NOT NULL,
                        created_at TEXT NOT NULL,
                        FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
                    );

                    CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at);
                    CREATE INDEX IF NOT EXISTS idx_prompts_category ON prompts(category);
                    CREATE INDEX IF NOT EXISTS idx_prompts_content_hash ON prompts(content_hash);
                    CREATE INDEX IF NOT EXISTS idx_prompts_status ON prompts(status);
                    CREATE INDEX IF NOT EXISTS idx_quality_scores_prompt_id ON quality_scores(prompt_id);
                    CREATE INDEX IF NOT EXISTS idx_efficiency_metrics_prompt_id ON efficiency_metrics(prompt_id);
                    CREATE INDEX IF NOT EXISTS idx_version_history_prompt_id ON version_history(prompt_id);
                "#,
                )
                .map_err(|e| {
                    PromptTrackingError::DatabaseError(format!("Failed to run migration v1: {}", e))
                })?;
        }

        Ok(())
    }

    /// Migration v2: Add full-text search support
    fn migration_v2(&self) -> Result<()> {
        // Add FTS table for full-text search
        self.conn
            .execute_batch(
                r#"
                -- Create FTS5 virtual table for full-text search
                CREATE VIRTUAL TABLE IF NOT EXISTS prompts_fts USING fts5(
                    content,
                    category,
                    content='prompts',
                    content_rowid='rowid'
                );

                -- Create triggers to keep FTS table in sync
                CREATE TRIGGER IF NOT EXISTS prompts_fts_insert AFTER INSERT ON prompts BEGIN
                    INSERT INTO prompts_fts(rowid, content, category)
                    VALUES (NEW.rowid, NEW.content, COALESCE(NEW.category, ''));
                END;

                CREATE TRIGGER IF NOT EXISTS prompts_fts_delete AFTER DELETE ON prompts BEGIN
                    INSERT INTO prompts_fts(prompts_fts, rowid, content, category)
                    VALUES ('delete', OLD.rowid, OLD.content, COALESCE(OLD.category, ''));
                END;

                CREATE TRIGGER IF NOT EXISTS prompts_fts_update AFTER UPDATE ON prompts BEGIN
                    INSERT INTO prompts_fts(prompts_fts, rowid, content, category)
                    VALUES ('delete', OLD.rowid, OLD.content, COALESCE(OLD.category, ''));
                    INSERT INTO prompts_fts(rowid, content, category)
                    VALUES (NEW.rowid, NEW.content, COALESCE(NEW.category, ''));
                END;
                "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to run migration v2: {}", e))
            })?;

        Ok(())
    }

    /// Get migration history
    pub fn get_migration_history(&self) -> Result<Vec<(i32, String)>> {
        self.ensure_schema_version_table()?;

        let mut stmt = self
            .conn
            .prepare("SELECT version, applied_at FROM schema_version ORDER BY version")
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        let history = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get migration history: {}", e))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to collect history: {}", e))
            })?;

        Ok(history)
    }

    /// Check if migrations are needed
    pub fn needs_migration(&self) -> Result<bool> {
        self.ensure_schema_version_table()?;
        let current = self.get_current_version()?;
        Ok(current < CURRENT_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_migration_manager_creation() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);
        assert!(manager.ensure_schema_version_table().is_ok());
    }

    #[test]
    fn test_get_current_version_empty() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);
        manager.ensure_schema_version_table().unwrap();

        let version = manager.get_current_version().unwrap();
        assert_eq!(version, 0);
    }

    #[test]
    fn test_set_and_get_version() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);
        manager.ensure_schema_version_table().unwrap();

        manager.set_version(1).unwrap();
        let version = manager.get_current_version().unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_run_migrations() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);

        assert!(manager.run_migrations().is_ok());

        let version = manager.get_current_version().unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn test_needs_migration() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);

        assert!(manager.needs_migration().unwrap());

        manager.run_migrations().unwrap();

        assert!(!manager.needs_migration().unwrap());
    }

    #[test]
    fn test_migration_history() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);

        manager.run_migrations().unwrap();

        let history = manager.get_migration_history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, 1);
        assert_eq!(history[1].0, 2);
    }

    #[test]
    fn test_idempotent_migrations() {
        let conn = create_test_db();
        let manager = MigrationManager::new(&conn);

        // Run migrations twice
        manager.run_migrations().unwrap();
        manager.run_migrations().unwrap();

        // Should still be at current version
        let version = manager.get_current_version().unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }
}
