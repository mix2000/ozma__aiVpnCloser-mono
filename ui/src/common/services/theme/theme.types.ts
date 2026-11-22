/* Выбор темы: как в системе или явно. */
export type ThemeMode = 'system' | 'light' | 'dark';

/* Тема, которая применена сейчас. */
export type ResolvedTheme = 'light' | 'dark';

/* Значение контекста темы. */
export interface IThemeContextValue {
  /* Что выбрано. */
  mode: ThemeMode;
  /* Что применено (для `system` — по настройке ОС). */
  resolved: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}
