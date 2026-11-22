import { act, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { installFallback, renderHookWithProviders } from 'common/test';

import { useDiscovery } from './hooks';

describe('useDiscovery', () => {
  it('загружает и обновляет по кнопке', async () => {
    const call = vi.fn().mockResolvedValue({ interfaces: [], processes: [], systemVpns: [] });
    installFallback({ call, listen: () => () => undefined });
    const { result } = renderHookWithProviders(() => useDiscovery());
    await waitFor(() => expect(result.current.found).not.toBeNull());
    expect(result.current.loadError).toBeNull();
    await act(async () => {
      await result.current.refresh();
    });
    expect(call).toHaveBeenCalledTimes(2);
    expect(call).toHaveBeenCalledWith('discover', undefined);
  });
});
