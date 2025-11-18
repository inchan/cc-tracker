//! Database operations and management
//!
//! Provides SQLite database operations for storing and retrieving prompts.

use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::Path;

use crate::models::{EfficiencyMetrics, Prompt, PromptMetadata, QualityScore};
use crate::{PromptTrackingError, Result};

/// Database manager for prompt storage
pub struct Database {
    conn: Connection,
}

/// Filter options for querying prompts
#[derive(Debug, Default, Clone)]
pub struct PromptFilter {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search_query: Option<String>,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &str) -> Result<Self> {
        let path = shellexpand::tilde(path).to_string();

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to create directory: {}", e))
            })?;
        }

        let conn = Connection::open(&path).map_err(|e| {
            PromptTrackingError::DatabaseError(format!("Failed to open database: {}", e))
        })?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            PromptTrackingError::DatabaseError(format!("Failed to create in-memory database: {}", e))
        })?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                r#"
            -- Prompts table
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                category TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                model TEXT NOT NULL,
                input_tokens INTEGER,
                output_tokens INTEGER,
                execution_time_ms INTEGER,
                estimated_cost REAL,
                context TEXT
            );

            -- Tags table
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            );

            -- Prompt-Tags junction table
            CREATE TABLE IF NOT EXISTS prompt_tags (
                prompt_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (prompt_id, tag_id),
                FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            -- Quality scores table
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

            -- Efficiency metrics table
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

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at);
            CREATE INDEX IF NOT EXISTS idx_prompts_category ON prompts(category);
            CREATE INDEX IF NOT EXISTS idx_prompts_content_hash ON prompts(content_hash);
            CREATE INDEX IF NOT EXISTS idx_quality_scores_prompt_id ON quality_scores(prompt_id);
            CREATE INDEX IF NOT EXISTS idx_efficiency_metrics_prompt_id ON efficiency_metrics(prompt_id);
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to initialize schema: {}", e))
            })?;

        Ok(())
    }

    /// Create a new prompt
    pub fn create_prompt(&self, prompt: &Prompt) -> Result<()> {
        self.conn
            .execute(
                r#"
            INSERT INTO prompts (
                id, content, content_hash, category, created_at, updated_at,
                model, input_tokens, output_tokens, execution_time_ms, estimated_cost, context
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
                params![
                    prompt.id,
                    prompt.content,
                    prompt.content_hash,
                    prompt.category,
                    prompt.created_at.to_rfc3339(),
                    prompt.updated_at.to_rfc3339(),
                    prompt.metadata.model,
                    prompt.metadata.input_tokens,
                    prompt.metadata.output_tokens,
                    prompt.metadata.execution_time_ms.map(|v| v as i64),
                    prompt.metadata.estimated_cost,
                    prompt.metadata.context,
                ],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to create prompt: {}", e))
            })?;

        // Insert tags
        for tag in &prompt.tags {
            self.add_tag_to_prompt(&prompt.id, tag)?;
        }

        Ok(())
    }

    /// Get a prompt by ID
    pub fn get_prompt(&self, id: &str) -> Result<Option<Prompt>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT id, content, content_hash, category, created_at, updated_at,
                   model, input_tokens, output_tokens, execution_time_ms, estimated_cost, context
            FROM prompts WHERE id = ?1
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        let prompt = stmt
            .query_row(params![id], |row| {
                Ok(self.row_to_prompt(row)?)
            })
            .optional()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get prompt: {}", e))
            })?;

        // Get tags if prompt exists
        if let Some(mut p) = prompt {
            p.tags = self.get_tags_for_prompt(&p.id)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }

    /// Update an existing prompt
    pub fn update_prompt(&self, prompt: &Prompt) -> Result<()> {
        self.conn
            .execute(
                r#"
            UPDATE prompts SET
                content = ?2, content_hash = ?3, category = ?4, updated_at = ?5,
                model = ?6, input_tokens = ?7, output_tokens = ?8,
                execution_time_ms = ?9, estimated_cost = ?10, context = ?11
            WHERE id = ?1
            "#,
                params![
                    prompt.id,
                    prompt.content,
                    prompt.content_hash,
                    prompt.category,
                    prompt.updated_at.to_rfc3339(),
                    prompt.metadata.model,
                    prompt.metadata.input_tokens,
                    prompt.metadata.output_tokens,
                    prompt.metadata.execution_time_ms.map(|v| v as i64),
                    prompt.metadata.estimated_cost,
                    prompt.metadata.context,
                ],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to update prompt: {}", e))
            })?;

        // Update tags
        self.remove_all_tags_from_prompt(&prompt.id)?;
        for tag in &prompt.tags {
            self.add_tag_to_prompt(&prompt.id, tag)?;
        }

        Ok(())
    }

    /// Delete a prompt by ID
    pub fn delete_prompt(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM prompts WHERE id = ?1", params![id])
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to delete prompt: {}", e))
            })?;

        Ok(())
    }

    /// List prompts with optional filtering
    pub fn list_prompts(&self, filter: &PromptFilter) -> Result<Vec<Prompt>> {
        let mut query = String::from(
            r#"
            SELECT DISTINCT p.id, p.content, p.content_hash, p.category, p.created_at, p.updated_at,
                   p.model, p.input_tokens, p.output_tokens, p.execution_time_ms, p.estimated_cost, p.context
            FROM prompts p
            "#,
        );

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Join with tags if filtering by tags
        if !filter.tags.is_empty() {
            query.push_str(
                r#"
                JOIN prompt_tags pt ON p.id = pt.prompt_id
                JOIN tags t ON pt.tag_id = t.id
                "#,
            );
        }

        // Category filter
        if let Some(ref category) = filter.category {
            conditions.push(format!("p.category = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(category.clone()));
        }

        // Tags filter
        if !filter.tags.is_empty() {
            let placeholders: Vec<String> = filter
                .tags
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", params_vec.len() + i + 1))
                .collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in &filter.tags {
                params_vec.push(Box::new(tag.clone()));
            }
        }

        // Search query
        if let Some(ref search) = filter.search_query {
            conditions.push(format!("p.content LIKE ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(format!("%{}%", search)));
        }

        // Add WHERE clause if there are conditions
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Order by created_at descending
        query.push_str(" ORDER BY p.created_at DESC");

        // Limit and offset
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = self.conn.prepare(&query).map_err(|e| {
            PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
        })?;

        let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let prompts: Vec<Prompt> = stmt
            .query_map(params_slice.as_slice(), |row| {
                self.row_to_prompt(row)
            })
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to list prompts: {}", e))
            })?
            .collect::<SqliteResult<Vec<Prompt>>>()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to collect prompts: {}", e))
            })?;

        // Get tags for each prompt
        let mut result = Vec::new();
        for mut prompt in prompts {
            prompt.tags = self.get_tags_for_prompt(&prompt.id)?;
            result.push(prompt);
        }

        Ok(result)
    }

    /// Search prompts by content
    pub fn search_prompts(&self, query: &str) -> Result<Vec<Prompt>> {
        let filter = PromptFilter {
            search_query: Some(query.to_string()),
            ..Default::default()
        };
        self.list_prompts(&filter)
    }

    /// Check if a prompt with the same hash exists
    pub fn find_by_hash(&self, hash: &str) -> Result<Option<Prompt>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT id, content, content_hash, category, created_at, updated_at,
                   model, input_tokens, output_tokens, execution_time_ms, estimated_cost, context
            FROM prompts WHERE content_hash = ?1
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        let prompt = stmt
            .query_row(params![hash], |row| self.row_to_prompt(row))
            .optional()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to find by hash: {}", e))
            })?;

        if let Some(mut p) = prompt {
            p.tags = self.get_tags_for_prompt(&p.id)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }

    /// Get total number of prompts
    pub fn count_prompts(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to count prompts: {}", e))
            })?;

        Ok(count as usize)
    }

    /// Save quality score for a prompt
    pub fn save_quality_score(&self, score: &QualityScore) -> Result<()> {
        self.conn
            .execute(
                r#"
            INSERT INTO quality_scores (
                prompt_id, total_score, clarity, completeness, specificity, guidance, analyzed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
                params![
                    score.prompt_id,
                    score.total_score,
                    score.clarity,
                    score.completeness,
                    score.specificity,
                    score.guidance,
                    score.analyzed_at.to_rfc3339(),
                ],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to save quality score: {}", e))
            })?;

        Ok(())
    }

    /// Get latest quality score for a prompt
    pub fn get_quality_score(&self, prompt_id: &str) -> Result<Option<QualityScore>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT prompt_id, total_score, clarity, completeness, specificity, guidance, analyzed_at
            FROM quality_scores WHERE prompt_id = ?1
            ORDER BY analyzed_at DESC LIMIT 1
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        stmt.query_row(params![prompt_id], |row| {
            Ok(QualityScore {
                prompt_id: row.get(0)?,
                total_score: row.get(1)?,
                clarity: row.get(2)?,
                completeness: row.get(3)?,
                specificity: row.get(4)?,
                guidance: row.get(5)?,
                analyzed_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })
        .optional()
        .map_err(|e| {
            PromptTrackingError::DatabaseError(format!("Failed to get quality score: {}", e))
        })
    }

    /// Save efficiency metrics for a prompt
    pub fn save_efficiency_metrics(&self, metrics: &EfficiencyMetrics) -> Result<()> {
        self.conn
            .execute(
                r#"
            INSERT INTO efficiency_metrics (
                prompt_id, efficiency_score, token_efficiency, time_efficiency, cost_efficiency, calculated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
                params![
                    metrics.prompt_id,
                    metrics.efficiency_score,
                    metrics.token_efficiency,
                    metrics.time_efficiency,
                    metrics.cost_efficiency,
                    metrics.calculated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!(
                    "Failed to save efficiency metrics: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Get latest efficiency metrics for a prompt
    pub fn get_efficiency_metrics(&self, prompt_id: &str) -> Result<Option<EfficiencyMetrics>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT prompt_id, efficiency_score, token_efficiency, time_efficiency, cost_efficiency, calculated_at
            FROM efficiency_metrics WHERE prompt_id = ?1
            ORDER BY calculated_at DESC LIMIT 1
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        stmt.query_row(params![prompt_id], |row| {
            Ok(EfficiencyMetrics {
                prompt_id: row.get(0)?,
                efficiency_score: row.get(1)?,
                token_efficiency: row.get(2)?,
                time_efficiency: row.get(3)?,
                cost_efficiency: row.get(4)?,
                calculated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
        })
        .optional()
        .map_err(|e| {
            PromptTrackingError::DatabaseError(format!("Failed to get efficiency metrics: {}", e))
        })
    }

    /// Get all quality scores for statistics
    pub fn get_all_quality_scores(&self) -> Result<Vec<QualityScore>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT prompt_id, total_score, clarity, completeness, specificity, guidance, analyzed_at
            FROM quality_scores ORDER BY analyzed_at DESC
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        let scores = stmt
            .query_map([], |row| {
                Ok(QualityScore {
                    prompt_id: row.get(0)?,
                    total_score: row.get(1)?,
                    clarity: row.get(2)?,
                    completeness: row.get(3)?,
                    specificity: row.get(4)?,
                    guidance: row.get(5)?,
                    analyzed_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                })
            })
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get quality scores: {}", e))
            })?
            .collect::<SqliteResult<Vec<QualityScore>>>()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to collect scores: {}", e))
            })?;

        Ok(scores)
    }

    // Helper methods

    fn row_to_prompt(&self, row: &rusqlite::Row) -> SqliteResult<Prompt> {
        Ok(Prompt {
            id: row.get(0)?,
            content: row.get(1)?,
            content_hash: row.get(2)?,
            category: row.get(3)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            tags: Vec::new(), // Will be populated separately
            metadata: PromptMetadata {
                model: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                execution_time_ms: row.get::<_, Option<i64>>(9)?.map(|v| v as u64),
                estimated_cost: row.get(10)?,
                context: row.get(11)?,
            },
        })
    }

    fn add_tag_to_prompt(&self, prompt_id: &str, tag: &str) -> Result<()> {
        // Insert tag if it doesn't exist
        self.conn
            .execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to insert tag: {}", e))
            })?;

        // Get tag ID
        let tag_id: i64 = self
            .conn
            .query_row("SELECT id FROM tags WHERE name = ?1", params![tag], |row| {
                row.get(0)
            })
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get tag ID: {}", e))
            })?;

        // Link tag to prompt
        self.conn
            .execute(
                "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                params![prompt_id, tag_id],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to link tag to prompt: {}", e))
            })?;

        Ok(())
    }

    fn get_tags_for_prompt(&self, prompt_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT t.name FROM tags t
            JOIN prompt_tags pt ON t.id = pt.tag_id
            WHERE pt.prompt_id = ?1
            "#,
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to prepare query: {}", e))
            })?;

        let tags = stmt
            .query_map(params![prompt_id], |row| row.get(0))
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to get tags: {}", e))
            })?
            .collect::<SqliteResult<Vec<String>>>()
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to collect tags: {}", e))
            })?;

        Ok(tags)
    }

    fn remove_all_tags_from_prompt(&self, prompt_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM prompt_tags WHERE prompt_id = ?1",
                params![prompt_id],
            )
            .map_err(|e| {
                PromptTrackingError::DatabaseError(format!("Failed to remove tags: {}", e))
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Prompt;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        assert_eq!(db.count_prompts().unwrap(), 0);
    }

    #[test]
    fn test_create_and_get_prompt() {
        let db = Database::in_memory().unwrap();
        let mut prompt = Prompt::new("Test content".to_string());
        prompt.content_hash = "test_hash".to_string();
        prompt.tags = vec!["tag1".to_string(), "tag2".to_string()];

        db.create_prompt(&prompt).unwrap();

        let retrieved = db.get_prompt(&prompt.id).unwrap().unwrap();
        assert_eq!(retrieved.content, prompt.content);
        assert_eq!(retrieved.tags.len(), 2);
    }

    #[test]
    fn test_update_prompt() {
        let db = Database::in_memory().unwrap();
        let mut prompt = Prompt::new("Original".to_string());
        prompt.content_hash = "hash1".to_string();

        db.create_prompt(&prompt).unwrap();

        prompt.content = "Updated".to_string();
        prompt.content_hash = "hash2".to_string();
        db.update_prompt(&prompt).unwrap();

        let retrieved = db.get_prompt(&prompt.id).unwrap().unwrap();
        assert_eq!(retrieved.content, "Updated");
    }

    #[test]
    fn test_delete_prompt() {
        let db = Database::in_memory().unwrap();
        let mut prompt = Prompt::new("To delete".to_string());
        prompt.content_hash = "hash".to_string();

        db.create_prompt(&prompt).unwrap();
        assert_eq!(db.count_prompts().unwrap(), 1);

        db.delete_prompt(&prompt.id).unwrap();
        assert_eq!(db.count_prompts().unwrap(), 0);
    }

    #[test]
    fn test_list_prompts_with_filter() {
        let db = Database::in_memory().unwrap();

        // Create prompts with different categories
        let mut prompt1 = Prompt::new("Prompt 1".to_string());
        prompt1.content_hash = "hash1".to_string();
        prompt1.category = Some("code".to_string());

        let mut prompt2 = Prompt::new("Prompt 2".to_string());
        prompt2.content_hash = "hash2".to_string();
        prompt2.category = Some("docs".to_string());

        db.create_prompt(&prompt1).unwrap();
        db.create_prompt(&prompt2).unwrap();

        // Filter by category
        let filter = PromptFilter {
            category: Some("code".to_string()),
            ..Default::default()
        };
        let results = db.list_prompts(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, Some("code".to_string()));
    }

    #[test]
    fn test_search_prompts() {
        let db = Database::in_memory().unwrap();

        let mut prompt = Prompt::new("Find this specific content".to_string());
        prompt.content_hash = "hash".to_string();
        db.create_prompt(&prompt).unwrap();

        let results = db.search_prompts("specific").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_by_hash() {
        let db = Database::in_memory().unwrap();

        let mut prompt = Prompt::new("Content".to_string());
        prompt.content_hash = "unique_hash".to_string();
        db.create_prompt(&prompt).unwrap();

        let found = db.find_by_hash("unique_hash").unwrap();
        assert!(found.is_some());

        let not_found = db.find_by_hash("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_quality_score_storage() {
        let db = Database::in_memory().unwrap();

        let mut prompt = Prompt::new("Content".to_string());
        prompt.content_hash = "hash".to_string();
        db.create_prompt(&prompt).unwrap();

        let score = QualityScore {
            prompt_id: prompt.id.clone(),
            total_score: 85.0,
            clarity: 25.0,
            completeness: 25.0,
            specificity: 20.0,
            guidance: 15.0,
            analyzed_at: chrono::Utc::now(),
        };

        db.save_quality_score(&score).unwrap();

        let retrieved = db.get_quality_score(&prompt.id).unwrap().unwrap();
        assert_eq!(retrieved.total_score, 85.0);
    }

    #[test]
    fn test_efficiency_metrics_storage() {
        let db = Database::in_memory().unwrap();

        let mut prompt = Prompt::new("Content".to_string());
        prompt.content_hash = "hash".to_string();
        db.create_prompt(&prompt).unwrap();

        let metrics = EfficiencyMetrics {
            prompt_id: prompt.id.clone(),
            efficiency_score: 75.0,
            token_efficiency: 80.0,
            time_efficiency: 70.0,
            cost_efficiency: 75.0,
            calculated_at: chrono::Utc::now(),
        };

        db.save_efficiency_metrics(&metrics).unwrap();

        let retrieved = db.get_efficiency_metrics(&prompt.id).unwrap().unwrap();
        assert_eq!(retrieved.efficiency_score, 75.0);
    }
}
