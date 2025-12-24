import { Component, Show } from 'solid-js';
import { useI18n } from '../i18n';
import type { ProjectListItem } from '../types/project';
import { formatLastOpened } from '../types/project';
import './ProjectCard.css';

interface ProjectCardProps {
  project: ProjectListItem;
  onOpen: (id: string) => void;
  onDelete: (id: string) => void;
}

const ProjectCard: Component<ProjectCardProps> = (props) => {
  const { t, locale } = useI18n();

  const handleOpen = () => {
    props.onOpen(props.project.id);
  };

  const handleDelete = (e: MouseEvent) => {
    e.stopPropagation();
    props.onDelete(props.project.id);
  };

  return (
    <div class="project-card" onClick={handleOpen}>
      <div class="project-card-thumbnail">
        <Show
          when={props.project.thumbnailBase64}
          fallback={
            <div class="project-card-thumbnail-placeholder">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="3" width="20" height="14" rx="2" />
                <line x1="8" y1="21" x2="16" y2="21" />
                <line x1="12" y1="17" x2="12" y2="21" />
              </svg>
            </div>
          }
        >
          <img src={`data:image/png;base64,${props.project.thumbnailBase64}`} alt={props.project.name} />
        </Show>
      </div>

      <div class="project-card-content">
        <h3 class="project-card-title">{props.project.name}</h3>
        
        <div class="project-card-info">
          <span class="project-card-engine">{props.project.engineDisplayName}</span>
          <span class="project-card-separator">•</span>
          <span class="project-card-last-opened">
            {formatLastOpened(props.project.lastOpened, locale())}
          </span>
        </div>

        <div class="project-card-path" title={props.project.path}>
          {props.project.path}
        </div>

        <div class="project-card-progress">
          <div class="project-card-progress-bar">
            <div
              class="project-card-progress-fill"
              style={{ width: `${props.project.translationProgress}%` }}
            />
          </div>
          <span class="project-card-progress-text">
            {props.project.translationProgress}%
          </span>
        </div>
      </div>

      <div class="project-card-actions">
        <button
          class="project-card-btn project-card-btn-open"
          onClick={handleOpen}
          title={t('projectList.openProject')}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </button>
        <button
          class="project-card-btn project-card-btn-delete"
          onClick={handleDelete}
          title={t('projectList.deleteProject')}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default ProjectCard;
