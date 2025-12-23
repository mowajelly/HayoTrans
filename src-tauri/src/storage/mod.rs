//! Storage module for configuration and database management
//!
//! This module handles:
//! - INI configuration file (stored next to exe)
//! - SQLite database for project data (stored next to exe)

pub mod config;
pub mod database;
pub mod project_store;

pub use config::AppConfig;
pub use database::Database;
pub use project_store::ProjectStore;
