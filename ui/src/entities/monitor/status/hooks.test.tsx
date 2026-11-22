import { act, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { installFallback, installMock, renderHookWithProviders } from 'common/test';

import { useAppState, useMonitoringToggle } from './hooks';

describe('состояние монитора', () => {
  it('первое состояние и пуш из бэкенда', async () => {
    const mock = installMock();
    await mock.call('save_config', { config: { ...mock.config, watch: [{ kind: 'route' }] } });
    const { result } = renderHookWithProviders(() => useAppState());
    await waitFor(() => expect(result.current.state?.status).toBe('paused'));
    await act(async () => {
      await mock.call('set_monitoring', { on: true });
    });
    await waitFor(() => expect(result.current.state?.status).toBe('ok'));
    act(() => mock.setVpn(false));
    await waitFor(() => expect(result.current.state?.status).toBe('countdown'));
    expect(result.current.state?.title).toBe('VPN пропал — закрою через 30 с');
  });

  it('ошибка связи', async () => {
    installFallback({ call: () => Promise.reject('нет ответа'), listen: () => () => undefined });
    const { result } = renderHookWithProviders(() => useAppState());
    await waitFor(() => expect(result.current.error).toBe('нет ответа'));
  });

  it('переключатель «Следить» кладёт новое состояние в кеш', async () => {
    installMock();
    const { result } = renderHookWithProviders(() => ({
      toggle: useMonitoringToggle(),
      app: useAppState(),
    }));
    await waitFor(() => expect(result.current.app.state?.monitoring).toBe(false));
    act(() => result.current.toggle.toggle(true));
    await waitFor(() => expect(result.current.app.state?.monitoring).toBe(true));
    expect(result.current.toggle.switching).toBe(false);
  });

  it('ошибка переключения — сообщение', async () => {
    installFallback({
      call: cmd => (cmd === 'set_monitoring' ? Promise.reject('нельзя') : Promise.reject('нет')),
      listen: () => () => undefined,
    });
    const { result } = renderHookWithProviders(() => useMonitoringToggle());
    act(() => result.current.toggle(true));
    await waitFor(() => expect(document.body.textContent).toContain('нельзя'));
  });
});
