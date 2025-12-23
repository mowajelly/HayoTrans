/**
 * API functions for application configuration
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Configuration from backend
 */
export interface AppConfig {
  language: string;
  theme: string;
  lastProjectId: string | null;
  windowWidth: number;
  windowHeight: number;
  maxLineLength: number | null;
}

/**
 * Get the current configuration from backend
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

/**
 * Set the UI language
 */
export async function setLanguage(language: string): Promise<void> {
  return invoke<void>('set_language', { language });
}

/**
 * Set the UI theme
 */
export async function setTheme(theme: string): Promise<void> {
  return invoke<void>('set_theme', { theme });
}

/**
 * Set the last opened project ID
 */
export async function setLastProject(projectId: string | null): Promise<void> {
  return invoke<void>('set_last_project', { projectId });
}

/**
 * Set the window size
 */
export async function setWindowSize(width: number, height: number): Promise<void> {
  return invoke<void>('set_window_size', { width, height });
}

/**
 * Get the application data path (where config and database are stored)
 */
export async function getAppDataPath(): Promise<string> {
  return invoke<string>('get_app_data_path');
}

/**
 * Open the application data folder in the system file explorer
 */
export async function openAppDataFolder(): Promise<void> {
  return invoke<void>('open_app_data_folder');
}
