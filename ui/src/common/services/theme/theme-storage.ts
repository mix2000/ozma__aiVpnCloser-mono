import type { ResolvedTheme, ThemeMode } from './theme.types';

/** Где лежит выбор темы (переключателя в окне пока нет — по умолчанию как в системе). */
export const THEME_STORAGE_KEY = 'ai-vpn-closer:theme-mode';

/** Атрибут на `<html>`, по которому ветвятся цвета в styles/global.scss. */
export const THEME_ATTRIBUTE = 'data-theme';

export const DEFAULT_THEME_MODE: ThemeMode = 'system';

const MODES: ThemeMode[] = ['system', 'light', 'dark'];
const DARK_QUERY = '(prefers-color-scheme: dark)';

/** Прочитать выбор; любой отказ хранилища — «выбора нет». */
export function readThemeMode(): ThemeMode {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    return MODES.includes(stored as ThemeMode) ? (stored as ThemeMode) : DEFAULT_THEME_MODE;
  } catch {
    return DEFAULT_THEME_MODE;
  }
}

export function writeThemeMode(mode: ThemeMode): void {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, mode);
  } catch {
    /* Не сохранилось — тема всё равно применится на эту сессию. */
  }
}

/** Что просит система. */
export function systemTheme(): ResolvedTheme {
  return window.matchMedia?.(DARK_QUERY).matches ? 'dark' : 'light';
}

/** Подписка на переключение темы в системе. */
export function subscribeSystemTheme(onChange: () => void): () => void {
  const mql = window.matchMedia?.(DARK_QUERY);
  mql?.addEventListener('change', onChange);
  return () => mql?.removeEventListener('change', onChange);
}

export function resolveTheme(mode: ThemeMode, system: ResolvedTheme): ResolvedTheme {
  return mode === 'system' ? system : mode;
}

/** Применить тему к документу. */
export function applyTheme(resolved: ResolvedTheme): void {
  document.documentElement.setAttribute(THEME_ATTRIBUTE, resolved);
}
