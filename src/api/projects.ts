/**
 * API functions for project management
 */

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ProjectInfo } from '../types/project';

/**
 * Get all saved projects
 */
export async function getProjects(): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>('get_projects');
}

/**
 * Add a new project by path
 */
export async function addProject(path: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>('add_project', { path });
}

/**
 * Get a project by ID
 */
export async function getProjectById(id: string): Promise<ProjectInfo | null> {
  const projects = await getProjects();
  return projects.find(p => p.id === id) || null;
}

/**
 * Delete a project by ID
 */
export async function deleteProject(id: string): Promise<void> {
  return invoke<void>('delete_project', { id });
}

/**
 * Open a project (updates last opened time)
 */
export async function openProject(id: string): Promise<ProjectInfo> {
  return invoke<ProjectInfo>('open_project', { id });
}

/**
 * Open folder dialog to select a game folder
 */
export async function selectGameFolder(): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    title: 'Select Game Folder',
  });
  
  return result as string | null;
}

/**
 * Detect game engine from folder path
 */
export async function detectGameEngine(path: string): Promise<ProjectInfo | null> {
  try {
    return await invoke<ProjectInfo>('detect_game_engine', { path });
  } catch {
    return null;
  }
}
