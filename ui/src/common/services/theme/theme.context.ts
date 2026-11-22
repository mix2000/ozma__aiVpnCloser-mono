import { createContext } from 'react';

import type { IThemeContextValue } from './theme.types';

/* Значение по умолчанию — рабочее: тема нужна и вне провайдера (тесты, ошибки). */
export const ThemeContext = createContext<IThemeContextValue>({
  mode: 'system',
  resolved: 'light',
  setMode: () => undefined,
});
