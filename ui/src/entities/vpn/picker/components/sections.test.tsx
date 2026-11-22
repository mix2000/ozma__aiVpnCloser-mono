import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { InterfacesSection } from './interfaces-section';
import { ProcessesSection } from './processes-section';
import { RouteSection } from './route-section';
import { SystemVpnsSection } from './system-vpns-section';

const none = new Set<string>();

describe('разделы окна выбора', () => {
  it('интерфейсы: загрузка, пусто, найденные, выбранные без интерфейса, «все»', async () => {
    const toggle = vi.fn();
    const onShowAll = vi.fn();
    const props = { selected: none, showAll: false, onShowAll, toggle };
    const { rerender } = render(
      <InterfacesSection loaded={false} ifaces={[]} missing={[]} {...props} />
    );
    expect(document.querySelector('.ant-spin')).not.toBeNull();
    rerender(<InterfacesSection loaded ifaces={[]} missing={[]} {...props} />);
    expect(screen.getByText('Туннельных интерфейсов не найдено.')).toBeInTheDocument();
    rerender(
      <InterfacesSection
        loaded
        ifaces={[{ name: 'utun9', addrs: [], up: false, tunnel: true, desc: 'WireGuard Tunnel' }]}
        missing={[{ kind: 'interface', name: 'utun233', addr: '172.19.0.1' }]}
        {...props}
      />
    );
    expect(screen.getByText('выключен')).toBeInTheDocument();
    expect(screen.getByText('нет адреса')).toBeInTheDocument();
    expect(screen.getByText('сейчас нет')).toBeInTheDocument();
    const [found, missing] = screen.getAllByRole('checkbox');
    await userEvent.click(found);
    expect(toggle).toHaveBeenCalledWith(
      { kind: 'interface', name: 'utun9', addr: undefined },
      true
    );
    await userEvent.click(missing);
    expect(toggle).toHaveBeenCalledWith(
      { kind: 'interface', name: 'utun233', addr: '172.19.0.1' },
      false
    );
    await userEvent.click(screen.getByRole('switch', { name: 'Показать все интерфейсы' }));
    expect(onShowAll).toHaveBeenCalledWith(true, expect.anything());
  });

  it('программы: пусто, незапущенная выбранная снимается, ввод вручную', async () => {
    const toggle = vi.fn();
    const props = { selected: none, showAll: false, onShowAll: vi.fn(), toggle };
    const { rerender } = render(<ProcessesSection loaded procs={[]} missing={[]} {...props} />);
    expect(screen.getByText('Похожих на VPN программ не найдено.')).toBeInTheDocument();
    rerender(
      <ProcessesSection
        loaded
        procs={[{ name: 'xray', pid: 1, knownVpn: true }]}
        missing={[{ kind: 'process', name: 'sing-box' }]}
        {...props}
      />
    );
    const [xray, singBox] = screen.getAllByRole('checkbox');
    await userEvent.click(xray);
    expect(toggle).toHaveBeenCalledWith({ kind: 'process', name: 'xray' }, true);
    await userEvent.click(singBox);
    expect(toggle).toHaveBeenCalledWith({ kind: 'process', name: 'sing-box' }, false);
    await userEvent.click(screen.getByRole('button', { name: 'Добавить' }));
    expect(toggle).toHaveBeenCalledTimes(2);
    await userEvent.type(screen.getByLabelText('Имя программы'), '  openvpn  ');
    await userEvent.click(screen.getByRole('button', { name: 'Добавить' }));
    expect(toggle).toHaveBeenLastCalledWith({ kind: 'process', name: 'openvpn' }, true);
  });

  it('системные VPN: пусто, подключён/отключён, выбор', async () => {
    const toggle = vi.fn();
    const { rerender } = render(<SystemVpnsSection vpns={[]} selected={none} toggle={toggle} />);
    expect(screen.getByText('В системе нет настроенных VPN.')).toBeInTheDocument();
    rerender(
      <SystemVpnsSection
        vpns={[
          { id: 'A', name: 'V2BOX', connected: false },
          { id: 'B', name: 'Happ Plus', connected: true },
        ]}
        selected={none}
        toggle={toggle}
      />
    );
    expect(screen.getByText('подключён')).toBeInTheDocument();
    await userEvent.click(screen.getAllByRole('checkbox')[1]);
    expect(toggle).toHaveBeenCalledWith({ kind: 'system_vpn', id: 'B', name: 'Happ Plus' }, true);
  });

  it('маршрут: без отметки хостов не видно', () => {
    render(
      <RouteSection
        on={false}
        onToggle={vi.fn()}
        hostsText=""
        hostsCount={0}
        onHostsText={vi.fn()}
        onResetHosts={vi.fn()}
      />
    );
    expect(screen.queryByLabelText('Адреса агентов')).not.toBeInTheDocument();
  });
});
