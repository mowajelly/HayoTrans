import { Component, createSignal, createResource, Show, For } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import { useI18n } from '../i18n';
import { getProjectById } from '../api/projects';
import type { ProjectInfo, ProgressState } from '../types/project';
import { PROGRESS_STATES, isAtOrPast } from '../types/project';
import './ProjectWorkspacePage.css';

// Workspace view types
export type WorkspaceView =
  | 'overview'
  | 'unpack'
  | 'extract'
  | 'metadata'
  | 'translate'
  | 'repack'
  | 'finalize';

const ProjectWorkspacePage: Component = () => {
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useI18n();
  
  const [currentView, setCurrentView] = createSignal<WorkspaceView>('overview');

  // Fetch project data
  const [project, { refetch }] = createResource(
    () => params.id,
    async (id) => {
      const result = await getProjectById(id);
      return result;
    }
  );

  const handleBack = () => {
    navigate('/');
  };

  // Get CSS class for progress section based on state
  const getSectionClass = (sectionState: ProgressState): string => {
    const current = project()?.progressState || 'initial';
    if (current === sectionState) return 'progress-section current';
    if (isAtOrPast(current, sectionState)) return 'progress-section completed';
    return 'progress-section pending';
  };

  // Get icon for progress state
  const getStateIcon = (state: ProgressState, current: ProgressState) => {
    if (isAtOrPast(current, state) && current !== state) {
      return (
        <svg class="state-icon done" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      );
    }
    if (current === state) {
      return (
        <svg class="state-icon current" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="4" fill="currentColor" />
        </svg>
      );
    }
    return (
      <svg class="state-icon pending" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="4" />
      </svg>
    );
  };

  return (
    <div class="workspace-page">
      {/* Header */}
      <header class="workspace-header">
        <button class="back-btn" onClick={handleBack} title={t('common.close')}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>
        <div class="header-title">
          <Show when={project()} fallback={<span>{t('common.loading')}</span>}>
            <h1>{project()!.name}</h1>
            <span class="header-engine">{project()!.engine.displayName}</span>
          </Show>
        </div>
      </header>

      <div class="workspace-container">
        {/* Main content area */}
        <main class="workspace-main">
          <Show 
            when={!project.loading && project()} 
            fallback={
              <div class="loading-state">
                <div class="spinner" />
                <span>{t('common.loading')}</span>
              </div>
            }
          >
            {/* Content based on current view */}
            <Show when={currentView() === 'overview'}>
              <div class="view-content overview-view">
                <h2>{t('workspace.overview')}</h2>
                <div class="overview-grid">
                  <div class="overview-card">
                    <h3>{t('workspace.projectInfo')}</h3>
                    <dl>
                      <dt>{t('workspace.path')}</dt>
                      <dd>{project()!.path}</dd>
                      <dt>{t('workspace.engine')}</dt>
                      <dd>{project()!.engine.displayName}</dd>
                      <dt>{t('workspace.totalLines')}</dt>
                      <dd>{project()!.totalLines.toLocaleString()}</dd>
                      <dt>{t('workspace.translatedLines')}</dt>
                      <dd>{project()!.translatedLines.toLocaleString()}</dd>
                    </dl>
                  </div>
                  <div class="overview-card">
                    <h3>{t('workspace.translationProgress')}</h3>
                    <div class="progress-circle">
                      <span class="progress-value">
                        {project()!.totalLines > 0
                          ? Math.round((project()!.translatedLines / project()!.totalLines) * 100)
                          : 0}%
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </Show>

            <Show when={currentView() === 'unpack'}>
              <div class="view-content unpack-view">
                <h2>{t('workspace.unpackAssets')}</h2>
                <p>{t('workspace.unpackDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.startUnpack')}
                </button>
              </div>
            </Show>

            <Show when={currentView() === 'extract'}>
              <div class="view-content extract-view">
                <h2>{t('workspace.extractDialogues')}</h2>
                <p>{t('workspace.extractDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.startExtract')}
                </button>
              </div>
            </Show>

            <Show when={currentView() === 'metadata'}>
              <div class="view-content metadata-view">
                <h2>{t('workspace.translateMetadata')}</h2>
                <p>{t('workspace.metadataDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.openMetadataEditor')}
                </button>
              </div>
            </Show>

            <Show when={currentView() === 'translate'}>
              <div class="view-content translate-view">
                <h2>{t('workspace.translateDialogues')}</h2>
                <p>{t('workspace.translateDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.openTranslationEditor')}
                </button>
              </div>
            </Show>

            <Show when={currentView() === 'repack'}>
              <div class="view-content repack-view">
                <h2>{t('workspace.repackAssets')}</h2>
                <p>{t('workspace.repackDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.startRepack')}
                </button>
              </div>
            </Show>

            <Show when={currentView() === 'finalize'}>
              <div class="view-content finalize-view">
                <h2>{t('workspace.finalize')}</h2>
                <p>{t('workspace.finalizeDescription')}</p>
                <button class="action-btn primary">
                  {t('workspace.createFinalBuild')}
                </button>
              </div>
            </Show>
          </Show>
        </main>

        {/* Right sidebar */}
        <aside class="workspace-sidebar">
          <Show when={project()}>
            {/* Project info section */}
            <div class="sidebar-section project-info-section">
              <div class="project-thumbnail">
                <Show
                  when={project()!.thumbnailBase64}
                  fallback={
                    <div class="thumbnail-placeholder">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="2" y="3" width="20" height="14" rx="2" />
                        <line x1="8" y1="21" x2="16" y2="21" />
                        <line x1="12" y1="17" x2="12" y2="21" />
                      </svg>
                    </div>
                  }
                >
                  <img 
                    src={`data:image/png;base64,${project()!.thumbnailBase64}`} 
                    alt={project()!.name} 
                  />
                </Show>
              </div>
              <h3 class="project-name">{project()!.name}</h3>
              <span class="project-engine">{project()!.engine.displayName}</span>
            </div>

            <hr class="sidebar-divider" />

            {/* Progress sections */}
            <div class="sidebar-section progress-sections">
              <h4 class="section-label">{t('workspace.workflowProgress')}</h4>

              {/* Section 1: Unpack */}
              <div class={getSectionClass('assets_unpacked')}>
                <div class="section-header">
                  {getStateIcon('assets_unpacked', project()!.progressState)}
                  <span>{t('workspace.step1Unpack')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('unpack')}
                >
                  {t('workspace.unpackAssets')}
                </button>
              </div>

              <hr class="sidebar-divider" />

              {/* Section 2: Extract */}
              <div class={getSectionClass('dialogues_extracted')}>
                <div class="section-header">
                  {getStateIcon('dialogues_extracted', project()!.progressState)}
                  <span>{t('workspace.step2Extract')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('extract')}
                >
                  {t('workspace.extractDialogues')}
                </button>
              </div>

              <hr class="sidebar-divider" />

              {/* Section 3: Metadata */}
              <div class={getSectionClass('metadata_translated')}>
                <div class="section-header">
                  {getStateIcon('metadata_translated', project()!.progressState)}
                  <span>{t('workspace.step3Metadata')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('metadata')}
                >
                  {t('workspace.translateMetadata')}
                </button>
              </div>

              <hr class="sidebar-divider" />

              {/* Section 4: Translate */}
              <div class={getSectionClass('dialogues_translated')}>
                <div class="section-header">
                  {getStateIcon('dialogues_translated', project()!.progressState)}
                  <span>{t('workspace.step4Translate')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('translate')}
                >
                  {t('workspace.translateDialogues')}
                </button>
              </div>

              <hr class="sidebar-divider" />

              {/* Section 5: Repack */}
              <div class={getSectionClass('assets_repacked')}>
                <div class="section-header">
                  {getStateIcon('assets_repacked', project()!.progressState)}
                  <span>{t('workspace.step5Repack')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('repack')}
                >
                  {t('workspace.repackAssets')}
                </button>
              </div>

              <hr class="sidebar-divider" />

              {/* Section 6: Finalize */}
              <div class={getSectionClass('finalized')}>
                <div class="section-header">
                  {getStateIcon('finalized', project()!.progressState)}
                  <span>{t('workspace.step6Finalize')}</span>
                </div>
                <button 
                  class="section-action-btn"
                  onClick={() => setCurrentView('finalize')}
                >
                  {t('workspace.finalize')}
                </button>
              </div>
            </div>
          </Show>
        </aside>
      </div>
    </div>
  );
};

export default ProjectWorkspacePage;
