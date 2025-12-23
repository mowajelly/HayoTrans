import { createContext, useContext, createSignal, ParentComponent, Accessor, Setter } from 'solid-js';
import en from './locales/en.json';
import ko from './locales/ko.json';

// Supported locales
export type Locale = 'en' | 'ko';

// Translation dictionary type
type TranslationDict = typeof en;

const translations: Record<Locale, TranslationDict> = {
  en,
  ko,
};

// Get nested value from object using dot notation
function getNestedValue(obj: Record<string, unknown>, path: string): string {
  const keys = path.split('.');
  let current: unknown = obj;
  
  for (const key of keys) {
    if (current && typeof current === 'object' && key in current) {
      current = (current as Record<string, unknown>)[key];
    } else {
      return path; // Return the path if translation not found
    }
  }
  
  return typeof current === 'string' ? current : path;
}

// Context type
interface I18nContextType {
  locale: Accessor<Locale>;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
  availableLocales: Locale[];
  isInitialized: Accessor<boolean>;
  setInitialized: Setter<boolean>;
}

const I18nContext = createContext<I18nContextType>();

// Validate locale
export function isValidLocale(locale: string): locale is Locale {
  return locale === 'en' || locale === 'ko';
}

export const I18nProvider: ParentComponent<{ initialLocale?: Locale }> = (props) => {
  // Use provided initial locale or default to 'en'
  const [locale, setLocaleSignal] = createSignal<Locale>(props.initialLocale || 'en');
  const [isInitialized, setInitialized] = createSignal(false);

  const setLocale = (newLocale: Locale) => {
    setLocaleSignal(newLocale);
  };

  const t = (key: string): string => {
    const dict = translations[locale()];
    return getNestedValue(dict as unknown as Record<string, unknown>, key);
  };

  const value: I18nContextType = {
    locale,
    setLocale,
    t,
    availableLocales: ['en', 'ko'],
    isInitialized,
    setInitialized,
  };

  return (
    <I18nContext.Provider value={value}>
      {props.children}
    </I18nContext.Provider>
  );
};

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within an I18nProvider');
  }
  return context;
}

// Helper hook for just the translation function
export function useTranslation() {
  const { t } = useI18n();
  return t;
}

export { translations };
