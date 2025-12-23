import { Component, createSignal, createResource, createEffect, For, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { useI18n } from '../i18n';
import ProjectCard from '../components/ProjectCard';
import { getProjects, addProject, deleteProject, selectGameFolder } from '../api/projects';
import { setLanguage as setBackendLanguage, openAppDataFolder } from '../api/config';
import type { ProjectListItem } from '../types/project';
import { toProjectListItem } from '../types/project';
import './ProjectListPage.css';

const ProjectListPage: Component = () => {
  const { t, locale, setLocale, availableLocales } = useI18n();
  const navigate = useNavigate();
  
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = createSignal<string | null>(null);

  // Fetch projects
  const [projects, { refetch }] = createResource(async () => {
    try {
      const projectInfos = await getProjects();
      return projectInfos.map(toProjectListItem);
    } catch (e) {
      console.error('Failed to load projects:', e);
      // Return empty array for now (backend not implemented yet)
      return [] as ProjectListItem[];
    }
  });

  const handleAddProject = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const folderPath = await selectGameFolder();
      if (folderPath) {
        await addProject(folderPath);
        refetch();
      }
    } catch (e) {
      console.error('Failed to add project:', e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenProject = (id: string) => {
    navigate(`/project/${id}`);
  };

  const handleDeleteClick = (id: string) => {
    setDeleteConfirmId(id);
  };

  const handleDeleteConfirm = async () => {
    const id = deleteConfirmId();
    if (!id) return;
    
    try {
      await deleteProject(id);
      refetch();
    } catch (e) {
      console.error('Failed to delete project:', e);
      setError(String(e));
    } finally {
      setDeleteConfirmId(null);
    }
  };

  const handleDeleteCancel = () => {
    setDeleteConfirmId(null);
  };

  // Sync locale to backend when it changes
  createEffect(() => {
    const currentLocale = locale();
    setBackendLanguage(currentLocale).catch(err => {
      console.error('Failed to sync language to backend:', err);
    });
  });

  const toggleLanguage = () => {
    const currentIndex = availableLocales.indexOf(locale());
    const nextIndex = (currentIndex + 1) % availableLocales.length;
    setLocale(availableLocales[nextIndex]);
  };

  const handleOpenDataFolder = async () => {
    try {
      await openAppDataFolder();
    } catch (e) {
      console.error('Failed to open data folder:', e);
      setError(String(e));
    }
  };

  return (
    <div class="project-list-page">
      <header class="project-list-header">
        <div class="header-left">
          <h1 class="app-title">{t('app.title')}</h1>
          <span class="app-subtitle">{t('app.subtitle')}</span>
        </div>
        <div class="header-right">
          <button class="lang-toggle" onClick={toggleLanguage} title={t('settings.language')}>
            {locale().toUpperCase()}
          </button>
          <button class="settings-btn" onClick={handleOpenDataFolder} title={t('settings.openDataFolder')}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
          </button>
          <button class="settings-btn" title={t('common.settings')}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </button>
        </div>
      </header>

      <main class="project-list-main">
        <div class="project-list-toolbar">
          <h2 class="section-title">{t('projectList.title')}</h2>
          <button
            class="add-project-btn"
            onClick={handleAddProject}
            disabled={isLoading()}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            <span>{t('projectList.addProject')}</span>
          </button>
        </div>

        <Show when={error()}>
          <div class="error-banner">
            <span>{error()}</span>
            <button onClick={() => setError(null)}>×</button>
          </div>
        </Show>

        <Show
          when={!projects.loading}
          fallback={
            <div class="loading-state">
              <div class="spinner" />
              <span>{t('common.loading')}</span>
            </div>
          }
        >
          <Show
            when={projects()?.length}
            fallback={
              <div class="empty-state">
                <div class="empty-state-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    <line x1="12" y1="11" x2="12" y2="17" />
                    <line x1="9" y1="14" x2="15" y2="14" />
                  </svg>
                </div>
                <h3>{t('projectList.noProjects')}</h3>
                <p>{t('projectList.selectFolder')}</p>
                <button
                  class="add-project-btn primary"
                  onClick={handleAddProject}
                  disabled={isLoading()}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                  <span>{t('projectList.addProject')}</span>
                </button>
              </div>
            }
          >
            <div class="project-list">
              <For each={projects()}>
                {(project) => (
                  <ProjectCard
                    project={project}
                    onOpen={handleOpenProject}
                    onDelete={handleDeleteClick}
                  />
                )}
              </For>
            </div>
          </Show>
        </Show>
      </main>

      {/* Delete confirmation modal */}
      <Show when={deleteConfirmId()}>
        <div class="modal-overlay" onClick={handleDeleteCancel}>
          <div class="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{t('projectList.confirmDelete')}</h3>
            <p>{t('projectList.confirmDeleteNote')}</p>
            <div class="modal-actions">
              <button class="modal-btn cancel" onClick={handleDeleteCancel}>
                {t('common.cancel')}
              </button>
              <button class="modal-btn danger" onClick={handleDeleteConfirm}>
                {t('projectList.deleteProject')}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default ProjectListPage;
