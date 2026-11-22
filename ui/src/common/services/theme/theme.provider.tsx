import { ConfigProvider } from 'antd';
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
  type ReactNode,
} from 'react';

import { darkTheme, lightTheme } from './antd-theme';
import { ThemeContext } from './theme.context';
import {
  applyTheme,
  readThemeMode,
  resolveTheme,
  subscribeSystemTheme,
  systemTheme,
  writeThemeMode,
} from './theme-storage';
import type { ThemeMode } from './theme.types';

interface IThemeProviderProps {
  children: ReactNode;
}

/** Светлая и тёмная тема: по системе (следим на лету) или явный выбор; antd — та же. */
export function ThemeProvider({ children }: IThemeProviderProps) {
  const [mode, setModeState] = useState<ThemeMode>(readThemeMode);
  const system = useSyncExternalStore(subscribeSystemTheme, systemTheme, () => 'light' as const);
  const resolved = resolveTheme(mode, system);

  useEffect(() => applyTheme(resolved), [resolved]);

  const setMode = useCallback((m: ThemeMode) => {
    writeThemeMode(m);
    setModeState(m);
  }, []);

  const value = useMemo(() => ({ mode, resolved, setMode }), [mode, resolved, setMode]);

  return (
    <ThemeContext.Provider value={value}>
      <ConfigProvider theme={resolved === 'dark' ? darkTheme : lightTheme}>
        {children}
      </ConfigProvider>
    </ThemeContext.Provider>
  );
}
