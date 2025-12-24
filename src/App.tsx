import { Router, Route } from '@solidjs/router';
import { createSignal, createEffect, Show, onMount } from 'solid-js';
import { I18nProvider, useI18n, isValidLocale, Locale } from './i18n';
import { getConfig, setLanguage as setBackendLanguage } from './api/config';
import SplashScreen, { LoadingTask } from './components/SplashScreen';
import ProjectListPage from './pages/ProjectListPage';
import ProjectWorkspacePage from './pages/ProjectWorkspacePage';
import './styles/global.css';

// App content that needs i18n context
function AppContent() {
  const { setLocale, setInitialized, isInitialized } = useI18n();
  const [tasks, setTasks] = createSignal<LoadingTask[]>([
    { id: 'config', label: 'Loading configuration...', status: 'pending' },
    { id: 'ready', label: 'Preparing application...', status: 'pending' },
  ]);
  const [currentTask, setCurrentTask] = createSignal<string>('');

  const updateTask = (id: string, updates: Partial<LoadingTask>) => {
    setTasks(prev => prev.map(task => 
      task.id === id ? { ...task, ...updates } : task
    ));
  };

  onMount(async () => {
    try {
      // Task 1: Load configuration
      updateTask('config', { status: 'loading' });
      setCurrentTask('Connecting to backend...');

      const config = await getConfig();
      
      // Apply locale from backend config
      if (isValidLocale(config.language)) {
        setLocale(config.language as Locale);
      }

      updateTask('config', { status: 'done', label: `Loaded configuration (${config.language})` });

      // Task 2: Prepare application
      updateTask('ready', { status: 'loading' });
      setCurrentTask('Initializing...');

      // Small delay for visual feedback
      await new Promise(resolve => setTimeout(resolve, 300));

      updateTask('ready', { status: 'done', label: 'Application ready' });
      setCurrentTask('');

      // Mark as initialized
      setInitialized(true);
    } catch (error) {
      console.error('Initialization error:', error);
      updateTask('config', { 
        status: 'error', 
        error: String(error) 
      });
      setCurrentTask('Error during initialization');

      // Still mark as initialized to show the app (with default settings)
      setTimeout(() => {
        setInitialized(true);
      }, 2000);
    }
  });

  return (
    <Show
      when={isInitialized()}
      fallback={<SplashScreen tasks={tasks()} currentTask={currentTask()} />}
    >
      <Router>
        <Route path="/" component={ProjectListPage} />
        <Route path="/project/:id" component={ProjectWorkspacePage} />
      </Router>
    </Show>
  );
}

// Updated ProjectListPage to sync language changes to backend
function ProjectListPageWithSync() {
  const { locale } = useI18n();

  // Sync locale changes to backend
  createEffect(() => {
    const currentLocale = locale();
    setBackendLanguage(currentLocale).catch(err => {
      console.error('Failed to sync language to backend:', err);
    });
  });

  return <ProjectListPage />;
}

function App() {
  return (
    <I18nProvider>
      <AppContent />
    </I18nProvider>
  );
}

export default App;
