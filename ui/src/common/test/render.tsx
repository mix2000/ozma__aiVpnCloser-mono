// Обёртка для тестов компонентов и хуков: свой QueryClient, antd App и
// подставной бэкенд (мок без таймеров или свой).
import { App as AntApp } from 'antd';
import { render, renderHook, type RenderOptions } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';

import { setFallbackBackend, type IFallbackBackend } from 'common/services/api';
import { MockBackend } from 'mocks/mock-backend';
import { createQueryClient, QueryProvider } from 'providers';

/** Подключить мок бэкенда (без отсчёта по таймеру) и вернуть его. */
export function installMock(mock = new MockBackend()): MockBackend {
  setFallbackBackend({
    call: (cmd, args) => mock.call(cmd, args),
    listen: <T,>(_event: string, cb: (payload: T) => void) => mock.subscribe(s => cb(s as T)),
  });
  return mock;
}

/** Подключить свой подставной бэкенд. */
export function installFallback(b: IFallbackBackend) {
  setFallbackBackend(b);
}

export function Providers({ children }: { children: ReactNode }) {
  return (
    <QueryProvider client={createQueryClient()}>
      <AntApp>{children}</AntApp>
    </QueryProvider>
  );
}

export function renderWithProviders(ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return render(ui, { wrapper: Providers, ...options });
}

export function renderHookWithProviders<T>(hook: () => T) {
  return renderHook(hook, { wrapper: Providers });
}
