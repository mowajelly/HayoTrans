import { Component, For } from 'solid-js';
import './SplashScreen.css';

export interface LoadingTask {
  id: string;
  label: string;
  status: 'pending' | 'loading' | 'done' | 'error';
  error?: string;
}

interface SplashScreenProps {
  tasks: LoadingTask[];
  currentTask?: string;
}

const SplashScreen: Component<SplashScreenProps> = (props) => {
  const getStatusIcon = (status: LoadingTask['status']) => {
    switch (status) {
      case 'pending':
        return (
          <span class="task-icon pending">○</span>
        );
      case 'loading':
        return (
          <span class="task-icon loading">
            <span class="spinner-small" />
          </span>
        );
      case 'done':
        return (
          <span class="task-icon done">✓</span>
        );
      case 'error':
        return (
          <span class="task-icon error">✗</span>
        );
    }
  };

  return (
    <div class="splash-screen">
      <div class="splash-content">
        <div class="splash-logo">
          <svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="50" cy="50" r="45" stroke="currentColor" stroke-width="4" fill="none" />
            <text x="50" y="60" text-anchor="middle" font-size="30" font-weight="bold" fill="currentColor">HT</text>
          </svg>
        </div>
        <h1 class="splash-title">HayoTrans</h1>
        <p class="splash-subtitle">Game Translation Tool</p>

        <div class="task-list">
          <For each={props.tasks}>
            {(task) => (
              <div class={`task-item ${task.status}`}>
                {getStatusIcon(task.status)}
                <span class="task-label">{task.label}</span>
                {task.error && (
                  <span class="task-error">{task.error}</span>
                )}
              </div>
            )}
          </For>
        </div>

        {props.currentTask && (
          <div class="current-task">
            {props.currentTask}
          </div>
        )}
      </div>
    </div>
  );
};

export default SplashScreen;
