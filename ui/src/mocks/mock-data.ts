// Данные мока: машина из ТЗ (sing-box TUN utun233, корпоративный utun4, V2BOX, Happ).
import type { IAgent, IAppState, IConfig, IDiscovery, ILimits, Status } from 'common/types';

export const MOCK_DISCOVERY: IDiscovery = {
  interfaces: [
    {
      name: 'en0',
      addrs: ['192.168.1.20/24', 'fe80::1/64'],
      up: true,
      tunnel: false,
      desc: 'Wi-Fi',
    },
    {
      name: 'utun233',
      addrs: ['172.19.0.1/30'],
      up: true,
      tunnel: true,
      owner: { name: 'sing-box', pid: 812, confidence: 'exact' },
    },
    {
      name: 'utun4',
      addrs: ['10.8.0.6/32'],
      up: true,
      tunnel: true,
      owner: { name: 'OpenVPN', confidence: 'likely' },
    },
    { name: 'utun0', addrs: ['fe80::ce81:b1c:bd2c:69e/64'], up: true, tunnel: true },
  ],
  processes: [
    { name: 'sing-box', pid: 812, knownVpn: true },
    { name: 'xray', pid: 911, knownVpn: true },
    { name: 'Happ', pid: 1204, knownVpn: true },
  ],
  systemVpns: [
    { id: 'A1B2-V2BOX', name: 'V2BOX', connected: false },
    { id: 'C3D4-HAPP', name: 'Happ Plus', connected: false },
  ],
};

export const MOCK_ROOTS: IAgent[] = [
  { target: 'claudeCode', pid: 4101, name: 'claude', frozen: false, root: true, rootPid: 4101 },
  { target: 'claudeCode', pid: 4177, name: 'claude', frozen: false, root: true, rootPid: 4177 },
  { target: 'claudeCode', pid: 5230, name: 'claude', frozen: false, root: true, rootPid: 5230 },
  { target: 'codexCli', pid: 6020, name: 'codex', frozen: false, root: true, rootPid: 6020 },
  { target: 'claudeDesktop', pid: 990, name: 'Claude', frozen: false, root: true, rootPid: 990 },
];

/** Потомки сессий: MCP-серверы и команды агента. */
export const MOCK_HELPERS: IAgent[] = [
  { target: 'claudeCode', pid: 4102, name: 'node', frozen: false, root: false, rootPid: 4101 },
  { target: 'claudeCode', pid: 4103, name: 'node', frozen: false, root: false, rootPid: 4101 },
  { target: 'claudeCode', pid: 4178, name: 'node', frozen: false, root: false, rootPid: 4177 },
  { target: 'codexCli', pid: 6021, name: 'bash', frozen: false, root: false, rootPid: 6020 },
];

/** Пределы — те же числа, что LIMITS в ядре (services/config_validation.rs). */
export const MOCK_LIMITS: ILimits = {
  pollMs: { min: 200, max: 60000 },
  confirmFailures: { min: 1, max: 20 },
  closeAfterSec: { min: 0, max: 600 },
  killGraceSec: { min: 0, max: 60 },
};

/** Проверка как в ядре (по MOCK_LIMITS); текст ошибки — тот же. */
export function mockValidate(c: IConfig): string | null {
  const l = MOCK_LIMITS;
  const out = (v: number, r: { min: number; max: number }) => v < r.min || v > r.max;
  if (out(c.pollMs, l.pollMs))
    return `Интервал опроса должен быть от ${l.pollMs.min} до ${l.pollMs.max} мс`;
  if (out(c.confirmFailures, l.confirmFailures))
    return `Число подтверждений должно быть от 1 до ${l.confirmFailures.max}`;
  if (out(c.closeAfterSec, l.closeAfterSec))
    return `«Закрыть через» — не больше ${l.closeAfterSec.max} секунд`;
  if (out(c.killGraceSec, l.killGraceSec))
    return `Ожидание перед принудительным закрытием — не больше ${l.killGraceSec.max} секунд`;
  return null;
}

/** Цвет и заголовок статуса — как status_view в ядре. */
export function mockStatusView(
  monitoring: boolean,
  status: Status,
  left: number | null
): Pick<IAppState, 'color' | 'title'> {
  if (!monitoring || status === 'paused') return { color: 'gray', title: 'Наблюдение выключено' };
  switch (status) {
    case 'ok':
      return { color: 'green', title: 'VPN включён' };
    case 'down':
      return { color: 'yellow', title: 'VPN пропадает…' };
    case 'countdown':
      return {
        color: 'yellow',
        title:
          left == null
            ? 'VPN пропал — скоро закрою агентов'
            : `VPN пропал — закрою через ${left} с`,
      };
    case 'closed':
      return { color: 'red', title: 'Агенты закрыты, VPN нет' };
    default:
      return { color: 'gray', title: 'Проверяю VPN…' };
  }
}
