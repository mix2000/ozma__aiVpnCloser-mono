import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { installFallback, renderWithProviders } from 'common/test';
import type { IAppState } from 'common/types';

import { JournalTab } from './journal-tab';

const state = { status: 'ok', lastChangeAt: null, monitoring: true } as IAppState;

function backend(events: unknown[] | string) {
  const call = vi.fn((cmd: string) =>
    typeof events === 'string'
      ? Promise.reject(events)
      : Promise.resolve(cmd === 'recent_events' ? events : null)
  );
  installFallback({ call, listen: () => () => undefined });
  return call;
}

describe('вкладка «Журнал»', () => {
  it('события с метками, «Обновить» перечитывает', async () => {
    const call = backend([
      { at: '2026-11-22T10:00:00Z', kind: 'vpn_down', message: 'VPN пропал: utun233' },
      { at: '2026-11-22T09:00:00Z', kind: 'info', message: 'Запуск' },
    ]);
    renderWithProviders(<JournalTab state={state} active />);
    expect(await screen.findByText('VPN пропал: utun233')).toBeInTheDocument();
    expect(screen.getByText('VPN пропал')).toBeInTheDocument();
    expect(call).toHaveBeenCalledWith('recent_events', { limit: 200 });
    await userEvent.click(screen.getByRole('button', { name: /Обновить/ }));
    await vi.waitFor(() => expect(call).toHaveBeenCalledTimes(2));
  });

  it('пусто и ошибка', async () => {
    backend([]);
    const { unmount } = renderWithProviders(<JournalTab state={state} active />);
    expect(await screen.findByText('Пока пусто')).toBeInTheDocument();
    unmount();
    backend('Не удалось прочитать журнал');
    renderWithProviders(<JournalTab state={state} active />);
    expect(await screen.findByText('Не удалось прочитать журнал')).toBeInTheDocument();
  });

  it('неактивная вкладка не читает', () => {
    const call = backend([]);
    renderWithProviders(<JournalTab state={state} active={false} />);
    expect(call).not.toHaveBeenCalled();
  });
});
