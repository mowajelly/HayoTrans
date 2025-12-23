//! SQLite database for project and translation data
//!
//! Database file is stored next to the executable.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;

const DB_FILENAME: &str = "hayotrans.db";

/// Database manager
pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    /// Get the database file path (next to executable)
    pub fn db_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join(DB_FILENAME)
    }

    /// Open or create the database
    pub fn open() -> Result<Self, String> {
        let path = Self::db_path();
        tracing::info!("Database file path: {:?}", path);
        
        let conn = Connection::open(&path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        tracing::info!("Database opened successfully");
        
        let db = Self {
            conn: Mutex::new(conn),
        };
        
        db.init_schema()?;
        
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        
        conn.execute_batch(r#"
            -- Projects table
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                engine_type TEXT NOT NULL,
                engine_version TEXT,
                created_at TEXT NOT NULL,
                last_opened_at TEXT NOT NULL,
                total_lines INTEGER DEFAULT 0,
                translated_lines INTEGER DEFAULT 0,
                thumbnail_path TEXT
            );

            -- Translation files table
            CREATE TABLE IF NOT EXISTS translation_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                source_hash TEXT,
                last_parsed_at TEXT,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                UNIQUE(project_id, file_path)
            );

            -- Translation units table
            CREATE TABLE IF NOT EXISTS translation_units (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                unit_id TEXT NOT NULL,
                original TEXT NOT NULL,
                translated TEXT,
                status TEXT DEFAULT 'pending',
                translator_type TEXT,
                context_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (file_id) REFERENCES translation_files(id) ON DELETE CASCADE,
                UNIQUE(file_id, unit_id)
            );

            -- Indexes for better query performance
            CREATE INDEX IF NOT EXISTS idx_translation_files_project ON translation_files(project_id);
            CREATE INDEX IF NOT EXISTS idx_translation_units_file ON translation_units(file_id);
            CREATE INDEX IF NOT EXISTS idx_translation_units_status ON translation_units(status);
        "#).map_err(|e| format!("Failed to init schema: {}", e))?;
        
        Ok(())
    }

    /// Execute a query with a callback
    pub fn with_connection<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&Connection) -> SqliteResult<T>,
    {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        f(&conn).map_err(|e| e.to_string())
    }

    /// Execute a query with a mutable callback
    pub fn with_connection_mut<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut Connection) -> SqliteResult<T>,
    {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        f(&mut conn).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_database_schema() {
        // Use temp file for testing
        let temp_path = std::env::temp_dir().join("test_hayotrans.db");
        if temp_path.exists() {
            fs::remove_file(&temp_path).ok();
        }

        let conn = Connection::open(&temp_path).unwrap();
        let db = Database {
            conn: Mutex::new(conn),
        };

        // Init schema should succeed
        assert!(db.init_schema().is_ok());

        // Verify tables exist
        let result = db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            )?;
            let tables: Vec<String> = stmt
                .query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(tables)
        });

        assert!(result.is_ok());
        let tables = result.unwrap();
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"translation_files".to_string()));
        assert!(tables.contains(&"translation_units".to_string()));

        // Cleanup
        fs::remove_file(&temp_path).ok();
    }
}
