import { act, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { installMock, renderHookWithProviders } from 'common/test';

import { useConfig } from './hooks';

describe('useConfig', () => {
  it('читает, сохраняет, откатывает при ошибке проверки', async () => {
    const mock = installMock();
    const { result } = renderHookWithProviders(() => useConfig());
    await waitFor(() => expect(result.current.config).not.toBeNull());
    expect(result.current.config?.dryRun).toBe(true);

    let ok = false;
    await act(async () => {
      ok = await result.current.save({ closeAfterSec: 10 });
    });
    expect(ok).toBe(true);
    await waitFor(() => expect(result.current.config?.closeAfterSec).toBe(10));
    expect(mock.config.closeAfterSec).toBe(10);

    await act(async () => {
      ok = await result.current.save({ closeAfterSec: 9999 });
    });
    expect(ok).toBe(false);
    await waitFor(() => expect(result.current.config?.closeAfterSec).toBe(10));
    await waitFor(() => expect(document.body.textContent).toContain('не больше 600 секунд'));
  });

  it('монитор поменял «Следить» — конфиг в курсе', async () => {
    installMock();
    const { result } = renderHookWithProviders(() => useConfig());
    await waitFor(() => expect(result.current.config).not.toBeNull());
    act(() => result.current.syncMonitoring(true, false));
    await waitFor(() =>
      expect(result.current.config).toMatchObject({ monitoring: true, dryRun: false })
    );
    act(() => result.current.syncMonitoring(true, false));
    expect(result.current.config?.monitoring).toBe(true);
  });
});
