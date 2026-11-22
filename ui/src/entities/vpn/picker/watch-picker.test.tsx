import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { installFallback, installMock, renderWithProviders } from 'common/test';
import { DEFAULT_CONFIG } from 'entities/settings';

import { WatchPicker } from './watch-picker';

describe('окно выбора «что считать VPN»', () => {
  beforeEach(() => {
    installMock();
  });

  it('найденное с владельцами туннелей', async () => {
    renderWithProviders(
      <WatchPicker config={DEFAULT_CONFIG} onCancel={vi.fn()} onSave={vi.fn()} />
    );
    expect(await screen.findByText('(sing-box)')).toBeInTheDocument();
    expect(screen.getByText('(вероятно OpenVPN)')).toBeInTheDocument();
    expect(screen.getByText('V2BOX')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Сохранить' })).toBeDisabled();
  });

  it('рекомендуемое + программа вручную → сохранить', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<WatchPicker config={DEFAULT_CONFIG} onCancel={vi.fn()} onSave={onSave} />);
    await userEvent.click(await screen.findByRole('button', { name: 'Выбрать рекомендуемое' }));
    await userEvent.type(screen.getByLabelText('Имя программы'), 'openvpn{Enter}');
    await userEvent.click(screen.getByRole('button', { name: 'Сохранить' }));
    expect(onSave).toHaveBeenCalledTimes(1);
    const [watch, hosts] = onSave.mock.calls[0];
    expect(watch).toEqual([
      { kind: 'interface', name: 'utun233', addr: '172.19.0.1/30' },
      { kind: 'interface', name: 'utun4', addr: '10.8.0.6/32' },
      { kind: 'route' },
      { kind: 'process', name: 'openvpn' },
    ]);
    expect(hosts).toHaveLength(6);
    expect(screen.getByText('не запущена')).toBeInTheDocument();
  });

  it('хосты: свой список и «вернуть стандартные»', async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<WatchPicker config={DEFAULT_CONFIG} onCancel={vi.fn()} onSave={onSave} />);
    await userEvent.click(screen.getAllByRole('checkbox')[0]);
    const hosts = screen.getByLabelText('Адреса агентов');
    await userEvent.clear(hosts);
    await userEvent.type(hosts, 'claude.ai');
    expect(screen.getByText('По одному адресу в строке. Адресов: 1.')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Сохранить' }));
    expect(onSave).toHaveBeenCalledWith([{ kind: 'route' }], ['claude.ai']);
    await userEvent.click(screen.getByRole('button', { name: 'Вернуть стандартные' }));
    expect(screen.getByText('По одному адресу в строке. Адресов: 6.')).toBeInTheDocument();
  });

  it('ошибка бэкенда показывается', async () => {
    installFallback({
      call: () => Promise.reject('Нет доступа к сети'),
      listen: () => () => undefined,
    });
    renderWithProviders(
      <WatchPicker config={DEFAULT_CONFIG} onCancel={vi.fn()} onSave={vi.fn()} />
    );
    expect(await screen.findByText('Нет доступа к сети')).toBeInTheDocument();
  });
});
