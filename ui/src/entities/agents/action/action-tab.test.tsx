import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { installMock, renderWithProviders } from 'common/test';
import type { IAppState } from 'common/types';
import { DEFAULT_CONFIG } from 'entities/settings';
import { MOCK_LIMITS } from 'mocks/mock-data';

import { ActionTab } from './action-tab';

const state: IAppState = {
  monitoring: true,
  dryRun: true,
  status: 'countdown',
  countdownLeftSec: 20,
  checks: [],
  agents: [
    { target: 'claudeCode', pid: 4101, name: 'claude', frozen: true, root: true, rootPid: 4101 },
  ],
  lastChangeAt: null,
  color: 'gray',
  title: 'Наблюдение выключено',
};

describe('вкладка «Действие»', () => {
  beforeEach(() => {
    installMock();
  });

  it('цели с числом сессий и список «кого затронет»', async () => {
    renderWithProviders(
      <ActionTab state={state} config={DEFAULT_CONFIG} limits={MOCK_LIMITS} save={vi.fn()} />
    );
    expect(await screen.findByText('— 3 сессии (+3 вспомогательных процесса)')).toBeInTheDocument();
    expect(screen.getAllByText('— сейчас не запущен')).toHaveLength(2);
    await userEvent.click(screen.getByRole('button', { name: 'Кого затронет сейчас (5 сессий)' }));
    expect(screen.getByText('Claude Code · claude · pid 4101')).toBeInTheDocument();
    expect(screen.getByText('· +2 вспомогательных процесса')).toBeInTheDocument();
    expect(screen.getByText('заморожен')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Скрыть список' }));
    expect(screen.queryByText('заморожен')).not.toBeInTheDocument();
  });

  it('переключатели и цели сохраняются', async () => {
    const save = vi.fn().mockResolvedValue(true);
    renderWithProviders(
      <ActionTab state={state} config={DEFAULT_CONFIG} limits={MOCK_LIMITS} save={save} />
    );
    await userEvent.click(screen.getByText('Grok'));
    expect(save).toHaveBeenCalledWith({ targets: { ...DEFAULT_CONFIG.targets, grok: false } });
    const switches = screen.getAllByRole('switch');
    await userEvent.click(switches[0]);
    expect(save).toHaveBeenCalledWith({ freezeImmediately: false });
    await userEvent.click(switches[1]);
    expect(save).toHaveBeenCalledWith({ blockWhileDown: false });
    await userEvent.click(switches[2]);
    expect(save).toHaveBeenCalledWith({ dryRun: false });
  });

  it('«Закрыть через»: поле и подсказка', () => {
    const save = vi.fn().mockResolvedValue(true);
    renderWithProviders(
      <ActionTab state={state} config={DEFAULT_CONFIG} limits={MOCK_LIMITS} save={save} />
    );
    expect(screen.getByText('Если VPN не вернётся за 30 с — закрою агентов.')).toBeInTheDocument();
    const input = screen.getByLabelText('Закрыть через, секунд');
    fireEvent.change(input, { target: { value: '0' } });
    fireEvent.blur(input);
    expect(save).toHaveBeenCalledWith({ closeAfterSec: 0 });
    expect(screen.getByText('Закрою сразу, без ожидания.')).toBeInTheDocument();
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(save).toHaveBeenLastCalledWith({ closeAfterSec: 0 });
  });

  it('предупреждение, когда консольные программы не выбраны', () => {
    const config = {
      ...DEFAULT_CONFIG,
      dryRun: false,
      targets: { ...DEFAULT_CONFIG.targets, claudeCode: false, codexCli: false, grok: false },
    };
    renderWithProviders(
      <ActionTab state={state} config={config} limits={MOCK_LIMITS} save={vi.fn()} />
    );
    expect(screen.getByText(/Не выбрано ни одной консольной программы/)).toBeInTheDocument();
  });
});
