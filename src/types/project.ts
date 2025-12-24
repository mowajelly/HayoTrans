/**
 * Project types for frontend
 */

// Game engine types - matches Rust backend
export type GameEngineType =
  | 'RPGMaker'
  | 'WolfRPG'
  | 'Tyrano'
  | 'KiriKiri'
  | 'RenPy'
  | 'NwJs'
  | 'Electron'
  | 'Unknown';

// RPG Maker version - matches Rust backend
export type RPGMakerVersion =
  | 'MV'
  | 'MZ'
  | 'VXAce'
  | 'VX'
  | 'XP'
  | 'Unknown';

// Progress state - matches Rust backend
export type ProgressState =
  | 'initial'
  | 'assets_unpacked'
  | 'dialogues_extracted'
  | 'metadata_translated'
  | 'dialogues_translated'
  | 'assets_repacked'
  | 'finalized';

// Progress state info for UI
export interface ProgressStateInfo {
  state: ProgressState;
  index: number;
  label: string;
}

// All progress states in order
export const PROGRESS_STATES: ProgressState[] = [
  'initial',
  'assets_unpacked',
  'dialogues_extracted',
  'metadata_translated',
  'dialogues_translated',
  'assets_repacked',
  'finalized',
];

// Get progress percentage (0 to 100)
export function getProgressPercentage(state: ProgressState): number {
  const index = PROGRESS_STATES.indexOf(state);
  if (index === -1) return 0;
  return Math.round((index / (PROGRESS_STATES.length - 1)) * 100);
}

// Check if a state is at or past another state
export function isAtOrPast(current: ProgressState, target: ProgressState): boolean {
  return PROGRESS_STATES.indexOf(current) >= PROGRESS_STATES.indexOf(target);
}

// Engine info for display
export interface EngineInfo {
  engineType: GameEngineType;
  version?: RPGMakerVersion;
  displayName: string;
}

// Project info stored in database
export interface ProjectInfo {
  id: string;
  name: string;
  path: string;
  engine: EngineInfo;
  createdAt: string; // ISO 8601
  lastOpenedAt: string; // ISO 8601
  totalLines: number;
  translatedLines: number;
  thumbnailBase64?: string;
  progressState: ProgressState;
}

// Project list item for display
export interface ProjectListItem {
  id: string;
  name: string;
  path: string;
  engineDisplayName: string;
  lastOpened: string;
  translationProgress: number; // 0-100 (lines translated)
  progressState: ProgressState;
  thumbnailBase64?: string;
}

// Convert ProjectInfo to ProjectListItem
export function toProjectListItem(project: ProjectInfo): ProjectListItem {
  const translationProgress = project.totalLines > 0
    ? Math.round((project.translatedLines / project.totalLines) * 100)
    : 0;

  return {
    id: project.id,
    name: project.name,
    path: project.path,
    engineDisplayName: project.engine.displayName,
    lastOpened: project.lastOpenedAt,
    translationProgress,
    progressState: project.progressState,
    thumbnailBase64: project.thumbnailBase64,
  };
}

// Format date for display
export function formatLastOpened(isoDate: string, locale: string): string {
  const date = new Date(isoDate);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (locale === 'ko') {
    if (minutes < 1) return '방금 전';
    if (minutes < 60) return `${minutes}분 전`;
    if (hours < 24) return `${hours}시간 전`;
    if (days < 7) return `${days}일 전`;
    return date.toLocaleDateString('ko-KR');
  } else {
    if (minutes < 1) return 'just now';
    if (minutes < 60) return `${minutes} minutes ago`;
    if (hours < 24) return `${hours} hours ago`;
    if (days < 7) return `${days} days ago`;
    return date.toLocaleDateString('en-US');
  }
}
