//! Tauri commands for project management

use crate::retriever::GameDetector;
use crate::storage::{Database, ProjectStore};
use crate::storage::project_store::ProjectInfo;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

/// Application state holding the database connection
pub struct AppState {
    pub db: Mutex<Database>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let db = Database::open()?;
        Ok(Self { db: Mutex::new(db) })
    }
}

/// Get all saved projects
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<ProjectInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let store = ProjectStore::new(&db);
    store.get_all()
}

/// Add a new project by path
#[tauri::command]
pub async fn add_project(path: String, state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    // Check if path exists
    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err("Path does not exist".to_string());
    }

    // Get the db lock
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let store = ProjectStore::new(&db);

    // Check if project already exists
    if let Some(existing) = store.get_by_path(&path)? {
        // Update last opened time and return existing
        store.update_last_opened(&existing.id)?;
        return store.get_by_id(&existing.id)?
            .ok_or_else(|| "Project not found".to_string());
    }

    // Detect game engine
    let detection_result = GameDetector::detect(path_obj);
    
    if !detection_result.success {
        return Err("Could not detect game engine. Please select a valid game folder.".to_string());
    }

    let game_project = detection_result.project
        .ok_or_else(|| "No project metadata available".to_string())?;
    
    // Get name first before moving engine
    let name = game_project.name();
    let engine = game_project.engine;

    // Add project to database
    store.add(&name, &path, &engine)
}

/// Delete a project by ID
#[tauri::command]
pub async fn delete_project(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let store = ProjectStore::new(&db);
    store.delete(&id)
}

/// Open a project (updates last opened time)
#[tauri::command]
pub async fn open_project(id: String, state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let store = ProjectStore::new(&db);
    
    store.update_last_opened(&id)?;
    
    store.get_by_id(&id)?
        .ok_or_else(|| "Project not found".to_string())
}

/// Detect game engine from a folder path (for preview before adding)
#[tauri::command]
pub async fn detect_engine(path: String) -> Result<String, String> {
    let path_obj = Path::new(&path);
    let detection_result = GameDetector::detect(path_obj);
    
    if detection_result.success {
        if let Some(project) = detection_result.project {
            Ok(project.engine.name())
        } else {
            Ok("Unknown".to_string())
        }
    } else {
        Ok("Unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        // This test would need a proper temp directory setup
        // For now just verify the struct compiles
    }
}
