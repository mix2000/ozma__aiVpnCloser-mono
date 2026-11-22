import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { installMock, renderWithProviders } from 'common/test';
import type { IAppState, IConfig } from 'common/types';
import { DEFAULT_CONFIG } from 'entities/settings';

import { WatchTab } from './watch-tab';

const state: IAppState = {
  monitoring: true,
  dryRun: true,
  status: 'ok',
  countdownLeftSec: null,
  checks: [
    {
      item: { kind: 'interface', name: 'utun233', addr: '172.19.0.1' },
      ok: true,
      label: 'Интерфейс utun233 (172.19.0.1)',
      detail: 'utun233 включён',
      owner: { name: 'sing-box', confidence: 'exact' },
    },
  ],
  agents: [],
  lastChangeAt: null,
  color: 'gray',
  title: 'Наблюдение выключено',
};

describe('вкладка «Наблюдение»', () => {
  beforeEach(() => {
    installMock();
  });

  it('ничего не выбрано — предложение выбрать', () => {
    renderWithProviders(
      <WatchTab state={{ ...state, checks: [] }} config={DEFAULT_CONFIG} save={vi.fn()} />
    );
    expect(screen.getByText('Что считать включённым VPN?')).toBeInTheDocument();
    expect(screen.getByText('Ничего не выбрано')).toBeInTheDocument();
  });

  it('проверки с владельцем туннеля и «ещё не проверено»', () => {
    const config: IConfig = {
      ...DEFAULT_CONFIG,
      watch: [{ kind: 'interface', name: 'utun233', addr: '172.19.0.1' }, { kind: 'route' }],
    };
    renderWithProviders(<WatchTab state={state} config={config} save={vi.fn()} />);
    expect(screen.getByText('Интерфейс utun233 (172.19.0.1)')).toBeInTheDocument();
    expect(screen.getByText('(sing-box)')).toBeInTheDocument();
    expect(screen.getByText('проверяю…')).toBeInTheDocument();
  });

  it('правило all/any сохраняется', async () => {
    const save = vi.fn().mockResolvedValue(true);
    renderWithProviders(<WatchTab state={state} config={DEFAULT_CONFIG} save={save} />);
    await userEvent.click(screen.getByText('Хотя бы один пункт в порядке'));
    expect(save).toHaveBeenCalledWith({ mode: 'any' });
  });

  it('«Выбрать…» открывает окно, «Отмена» закрывает', async () => {
    renderWithProviders(<WatchTab state={state} config={DEFAULT_CONFIG} save={vi.fn()} />);
    const card = screen.getByText('За чем слежу').closest('.ant-card') as HTMLElement;
    await userEvent.click(within(card).getByRole('button', { name: 'Выбрать…' }));
    expect(await screen.findByText('Что считать включённым VPN')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Отмена' }));
    await vi.waitFor(() =>
      expect(screen.queryByText('Что считать включённым VPN')).not.toBeInTheDocument()
    );
  });

  it('сохранение выбора закрывает окно', async () => {
    const save = vi.fn().mockResolvedValue(true);
    renderWithProviders(<WatchTab state={state} config={DEFAULT_CONFIG} save={save} />);
    await userEvent.click(screen.getAllByRole('button', { name: 'Выбрать…' })[0]);
    await userEvent.click(await screen.findByRole('button', { name: 'Выбрать рекомендуемое' }));
    await userEvent.click(screen.getByRole('button', { name: 'Сохранить' }));
    expect(save).toHaveBeenCalledWith(expect.objectContaining({ hosts: DEFAULT_CONFIG.hosts }));
    await vi.waitFor(() =>
      expect(screen.queryByText('Что считать включённым VPN')).not.toBeInTheDocument()
    );
  });

  it('значки проверок: есть, нет, неизвестно; названия до первой проверки', () => {
    const config: IConfig = {
      ...DEFAULT_CONFIG,
      watch: [
        { kind: 'process', name: 'xray' },
        { kind: 'system_vpn', id: 'A', name: 'V2BOX' },
        { kind: 'interface', name: 'utun4' },
      ],
    };
    const checks = [
      { item: config.watch[0], ok: false, label: 'Программа xray', detail: 'не запущен' },
    ];
    renderWithProviders(
      <WatchTab state={{ ...state, checks, monitoring: false }} config={config} save={vi.fn()} />
    );
    expect(screen.getByLabelText('нет')).toBeInTheDocument();
    expect(screen.getAllByLabelText('неизвестно')).toHaveLength(2);
    expect(screen.getByText('Системный VPN «V2BOX»')).toBeInTheDocument();
    expect(screen.getByText('Интерфейс utun4')).toBeInTheDocument();
  });
});
