import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { readThemeMode, resolveTheme, THEME_STORAGE_KEY, writeThemeMode } from './theme-storage';
import { ThemeContext } from './theme.context';
import { ThemeProvider } from './theme.provider';
import { useTheme } from './use-theme';

function Probe() {
  const { mode, resolved, setMode } = useTheme();
  return (
    <button onClick={() => setMode('dark')}>
      {mode}/{resolved}
    </button>
  );
}

describe('тема', () => {
  afterEach(() => window.localStorage.clear());

  it('по умолчанию — как в системе; выбор сохраняется', () => {
    expect(readThemeMode()).toBe('system');
    writeThemeMode('dark');
    expect(readThemeMode()).toBe('dark');
    window.localStorage.setItem(THEME_STORAGE_KEY, 'мусор');
    expect(readThemeMode()).toBe('system');
    expect(resolveTheme('system', 'dark')).toBe('dark');
    expect(resolveTheme('light', 'dark')).toBe('light');
  });

  it('провайдер применяет тему к документу', () => {
    render(
      <ThemeProvider>
        <Probe />
      </ThemeProvider>
    );
    expect(screen.getByRole('button')).toHaveTextContent('system/light');
    expect(document.documentElement.dataset.theme).toBe('light');
    act(() => screen.getByRole('button').click());
    expect(screen.getByRole('button')).toHaveTextContent('dark/dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
  });

  it('без провайдера — рабочее значение по умолчанию', () => {
    render(<Probe />);
    expect(screen.getByRole('button')).toHaveTextContent('system/light');
    act(() => screen.getByRole('button').click());
    expect(screen.getByRole('button')).toHaveTextContent('system/light');
    expect(ThemeContext).toBeDefined();
  });
});
