//! Project store for CRUD operations on projects

use super::Database;
use crate::types::engine::{GameEngine, RpgMakerVersion, V8Engine, KiriKiriVersion};
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Engine information for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub engine_type: String,
    pub version: Option<String>,
    pub display_name: String,
}

impl From<&GameEngine> for EngineInfo {
    fn from(engine: &GameEngine) -> Self {
        match engine {
            GameEngine::RpgMaker(version) => Self {
                engine_type: "RpgMaker".to_string(),
                version: Some(format!("{:?}", version)),
                display_name: match version {
                    RpgMakerVersion::MV => "RPG Maker MV".to_string(),
                    RpgMakerVersion::MZ => "RPG Maker MZ".to_string(),
                    RpgMakerVersion::VXAce => "RPG Maker VX Ace".to_string(),
                    RpgMakerVersion::VX => "RPG Maker VX".to_string(),
                    RpgMakerVersion::XP => "RPG Maker XP".to_string(),
                },
            },
            GameEngine::KiriKiri(version) => Self {
                engine_type: "KiriKiri".to_string(),
                version: Some(format!("{:?}", version)),
                display_name: match version {
                    KiriKiriVersion::KAG3 => "KiriKiri KAG3".to_string(),
                    KiriKiriVersion::Z => "KiriKiri Z".to_string(),
                },
            },
            GameEngine::V8Engine(v8type) => Self {
                engine_type: "V8Engine".to_string(),
                version: Some(format!("{:?}", v8type)),
                display_name: match v8type {
                    V8Engine::NwJs => "NW.js".to_string(),
                    V8Engine::Electron => "Electron".to_string(),
                    V8Engine::Generic => "V8 Engine".to_string(),
                },
            },
            GameEngine::Unknown => Self {
                engine_type: "Unknown".to_string(),
                version: None,
                display_name: "Unknown".to_string(),
            },
        }
    }
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub engine: EngineInfo,
    pub created_at: String,
    pub last_opened_at: String,
    pub total_lines: i64,
    pub translated_lines: i64,
    pub thumbnail_path: Option<String>,
}

/// Project store for database operations
pub struct ProjectStore<'a> {
    db: &'a Database,
}

impl<'a> ProjectStore<'a> {
    /// Create a new project store
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Get all projects
    pub fn get_all(&self) -> Result<Vec<ProjectInfo>, String> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, path, engine_type, engine_version, 
                        created_at, last_opened_at, total_lines, translated_lines, thumbnail_path
                 FROM projects
                 ORDER BY last_opened_at DESC"
            )?;

            let projects = stmt
                .query_map([], |row| {
                    let engine_type: String = row.get(3)?;
                    let engine_version: Option<String> = row.get(4)?;
                    
                    Ok(ProjectInfo {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        engine: EngineInfo {
                            engine_type: engine_type.clone(),
                            version: engine_version.clone(),
                            display_name: Self::make_display_name(&engine_type, engine_version.as_deref()),
                        },
                        created_at: row.get(5)?,
                        last_opened_at: row.get(6)?,
                        total_lines: row.get(7)?,
                        translated_lines: row.get(8)?,
                        thumbnail_path: row.get(9)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            Ok(projects)
        })
    }

    /// Get a project by ID
    pub fn get_by_id(&self, id: &str) -> Result<Option<ProjectInfo>, String> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, path, engine_type, engine_version,
                        created_at, last_opened_at, total_lines, translated_lines, thumbnail_path
                 FROM projects WHERE id = ?"
            )?;

            let result = stmt.query_row([id], |row| {
                let engine_type: String = row.get(3)?;
                let engine_version: Option<String> = row.get(4)?;
                
                Ok(ProjectInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    engine: EngineInfo {
                        engine_type: engine_type.clone(),
                        version: engine_version.clone(),
                        display_name: Self::make_display_name(&engine_type, engine_version.as_deref()),
                    },
                    created_at: row.get(5)?,
                    last_opened_at: row.get(6)?,
                    total_lines: row.get(7)?,
                    translated_lines: row.get(8)?,
                    thumbnail_path: row.get(9)?,
                })
            });

            match result {
                Ok(project) => Ok(Some(project)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Get a project by path
    pub fn get_by_path(&self, path: &str) -> Result<Option<ProjectInfo>, String> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, path, engine_type, engine_version,
                        created_at, last_opened_at, total_lines, translated_lines, thumbnail_path
                 FROM projects WHERE path = ?"
            )?;

            let result = stmt.query_row([path], |row| {
                let engine_type: String = row.get(3)?;
                let engine_version: Option<String> = row.get(4)?;
                
                Ok(ProjectInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    engine: EngineInfo {
                        engine_type: engine_type.clone(),
                        version: engine_version.clone(),
                        display_name: Self::make_display_name(&engine_type, engine_version.as_deref()),
                    },
                    created_at: row.get(5)?,
                    last_opened_at: row.get(6)?,
                    total_lines: row.get(7)?,
                    translated_lines: row.get(8)?,
                    thumbnail_path: row.get(9)?,
                })
            });

            match result {
                Ok(project) => Ok(Some(project)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Add a new project
    pub fn add(&self, name: &str, path: &str, engine: &GameEngine) -> Result<ProjectInfo, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let engine_info = EngineInfo::from(engine);

        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO projects (id, name, path, engine_type, engine_version, created_at, last_opened_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, name, path, engine_info.engine_type, engine_info.version, now, now],
            )?;

            Ok(ProjectInfo {
                id,
                name: name.to_string(),
                path: path.to_string(),
                engine: engine_info,
                created_at: now.clone(),
                last_opened_at: now,
                total_lines: 0,
                translated_lines: 0,
                thumbnail_path: None,
            })
        })
    }

    /// Delete a project by ID
    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.db.with_connection(|conn| {
            conn.execute("DELETE FROM projects WHERE id = ?", [id])?;
            Ok(())
        })
    }

    /// Update last opened time
    pub fn update_last_opened(&self, id: &str) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE projects SET last_opened_at = ? WHERE id = ?",
                params![now, id],
            )?;
            Ok(())
        })
    }

    /// Update translation progress
    pub fn update_progress(&self, id: &str, total: i64, translated: i64) -> Result<(), String> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE projects SET total_lines = ?, translated_lines = ? WHERE id = ?",
                params![total, translated, id],
            )?;
            Ok(())
        })
    }

    /// Helper to create display name from engine type and version
    fn make_display_name(engine_type: &str, version: Option<&str>) -> String {
        match engine_type {
            "RpgMaker" => {
                match version {
                    Some("MV") => "RPG Maker MV".to_string(),
                    Some("MZ") => "RPG Maker MZ".to_string(),
                    Some("VXAce") => "RPG Maker VX Ace".to_string(),
                    Some("VX") => "RPG Maker VX".to_string(),
                    Some("XP") => "RPG Maker XP".to_string(),
                    _ => "RPG Maker".to_string(),
                }
            }
            "KiriKiri" => {
                match version {
                    Some("KAG3") => "KiriKiri KAG3".to_string(),
                    Some("Z") => "KiriKiri Z".to_string(),
                    _ => "KiriKiri".to_string(),
                }
            }
            "V8Engine" => {
                match version {
                    Some("NwJs") => "NW.js".to_string(),
                    Some("Electron") => "Electron".to_string(),
                    _ => "V8 Engine".to_string(),
                }
            }
            _ => "Unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn create_test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database { conn: Mutex::new(conn) };
        
        // Init schema manually for in-memory test
        db.with_connection(|conn| {
            conn.execute_batch(r#"
                CREATE TABLE projects (
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
            "#)?;
            Ok(())
        }).unwrap();
        
        db
    }

    #[test]
    fn test_add_and_get_project() {
        let db = create_test_db();
        let store = ProjectStore::new(&db);

        let engine = GameEngine::RpgMaker(RpgMakerVersion::MV);
        let project = store.add("Test Game", "/path/to/game", &engine).unwrap();

        assert_eq!(project.name, "Test Game");
        assert_eq!(project.path, "/path/to/game");
        assert_eq!(project.engine.engine_type, "RpgMaker");
        assert_eq!(project.engine.version, Some("MV".to_string()));

        // Get by ID
        let found = store.get_by_id(&project.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Game");

        // Get by path
        let found = store.get_by_path("/path/to/game").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_get_all_projects() {
        let db = create_test_db();
        let store = ProjectStore::new(&db);

        store.add("Game 1", "/path/1", &GameEngine::RpgMaker(RpgMakerVersion::MV)).unwrap();
        store.add("Game 2", "/path/2", &GameEngine::RpgMaker(RpgMakerVersion::MZ)).unwrap();

        let projects = store.get_all().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let db = create_test_db();
        let store = ProjectStore::new(&db);

        let project = store.add("Test", "/path", &GameEngine::Unknown).unwrap();
        
        store.delete(&project.id).unwrap();
        
        let found = store.get_by_id(&project.id).unwrap();
        assert!(found.is_none());
    }
}
