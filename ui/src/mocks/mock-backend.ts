// Мок бэкенда для разработки в браузере (npm run dev) и для тестов.
// Ведёт себя как Engine: держит конфиг, считает статус по флажкам «VPN есть»,
// ведёт отсчёт. Никаких реальных процессов и сети.
import type { IAgent, IAppState, ICheck, IConfig, IEventRecord, WatchItem } from 'common/types';
import { DEFAULT_CONFIG } from 'entities/settings';
import { watchTitle } from 'entities/vpn';

import {
  MOCK_DISCOVERY,
  MOCK_HELPERS,
  MOCK_LIMITS,
  MOCK_ROOTS,
  mockStatusView,
  mockValidate,
} from './mock-data';

type Listener = (s: IAppState) => void;

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

export class MockBackend {
  config: IConfig = clone(DEFAULT_CONFIG);
  /** «VPN есть» — переключается из консоли: window.__mock.setVpn(false). */
  vpnUp = true;
  private status: IAppState['status'] = 'unknown';
  private countdown: number | null = null;
  private lastChangeAt: string | null = null;
  private agents: IAgent[] = clone(MOCK_ROOTS);
  private events: IEventRecord[] = [];
  private listeners = new Set<Listener>();
  private timer: ReturnType<typeof setInterval> | null = null;

  constructor(opts: { tick?: boolean } = {}) {
    this.log('info', 'Программа запущена (мок).');
    this.recompute();
    if (opts.tick) this.timer = setInterval(() => this.tick(), 1000);
  }

  dispose() {
    if (this.timer) clearInterval(this.timer);
  }

  private log(kind: string, message: string) {
    this.events.unshift({ at: new Date().toISOString(), kind, message });
    this.events = this.events.slice(0, 500);
  }

  private checks(): ICheck[] {
    return this.config.watch.map(item => ({ label: watchTitle(item), ...this.check(item) }));
  }

  private check(item: WatchItem): Omit<ICheck, 'label'> {
    switch (item.kind) {
      case 'interface':
        return {
          item,
          owner: MOCK_DISCOVERY.interfaces.find(i => i.name === item.name)?.owner ?? null,
          ok: this.vpnUp,
          detail: this.vpnUp
            ? `${item.name} работает, адрес ${item.addr ?? '—'}`
            : `${item.name} не найден`,
        };
      case 'process':
        return { item, ok: this.vpnUp, detail: this.vpnUp ? 'запущен' : 'не запущен' };
      case 'system_vpn':
        return { item, ok: false, detail: 'не подключён' };
      case 'route':
        return {
          item,
          ok: this.vpnUp,
          detail: this.vpnUp
            ? 'все 6 хостов идут через utun233'
            : 'api.anthropic.com идёт через en0',
        };
    }
  }

  private vpnOk(): boolean {
    const c = this.checks();
    if (c.length === 0) return false;
    return this.config.mode === 'all' ? c.every(x => x.ok) : c.some(x => x.ok);
  }

  private setStatus(s: IAppState['status']) {
    if (s !== this.status) {
      this.status = s;
      this.lastChangeAt = new Date().toISOString();
    }
  }

  private recompute() {
    if (!this.config.monitoring) {
      this.setStatus('paused');
      this.countdown = null;
      return;
    }
    if (this.config.watch.length === 0) {
      this.setStatus('unknown');
      return;
    }
    const ok = this.vpnOk();
    if (ok) {
      if (this.status === 'countdown' || this.status === 'closed' || this.status === 'down') {
        this.log('vpn_up', 'VPN вернулся — агенты продолжают.');
        this.agents.forEach(a => (a.frozen = false));
      }
      this.setStatus('ok');
      this.countdown = null;
    } else if (this.status !== 'countdown' && this.status !== 'closed') {
      const dry = this.config.dryRun ? ' бы' : '';
      if (this.config.freezeImmediately && !this.config.dryRun)
        this.agents.forEach(a => (a.frozen = true));
      this.log(
        'vpn_down',
        `VPN пропал — заморозил${dry} агентов, закрою через ${this.config.closeAfterSec} с.`
      );
      this.countdown = this.config.closeAfterSec;
      this.setStatus('countdown');
    }
  }

  private tick() {
    if (this.status === 'countdown' && this.countdown != null) {
      this.countdown -= 1;
      if (this.countdown <= 0) {
        const n = this.agents.length;
        this.log('close', `${this.config.dryRun ? 'Закрыл бы' : 'Закрыто'}: агентов ×${n}.`);
        if (!this.config.dryRun) this.agents = [];
        this.countdown = null;
        this.setStatus('closed');
      }
    }
    this.recompute();
    this.emit();
  }

  setVpn(up: boolean) {
    this.vpnUp = up;
    this.recompute();
    this.emit();
  }

  state(): IAppState {
    return {
      monitoring: this.config.monitoring,
      dryRun: this.config.dryRun,
      status: this.status,
      countdownLeftSec: this.countdown,
      checks: this.checks(),
      agents: clone(this.agents),
      lastChangeAt: this.lastChangeAt,
      ...mockStatusView(this.config.monitoring, this.status, this.countdown),
    };
  }

  private emit() {
    const s = this.state();
    this.listeners.forEach(l => l(s));
  }

  subscribe(l: Listener): () => void {
    this.listeners.add(l);
    return () => this.listeners.delete(l);
  }

  async call(cmd: string, args: Record<string, unknown> = {}): Promise<unknown> {
    switch (cmd) {
      case 'get_state':
        return this.state();
      case 'discover':
        return clone(MOCK_DISCOVERY);
      case 'get_limits':
        return clone(MOCK_LIMITS);
      case 'get_config':
        return clone(this.config);
      case 'save_config': {
        const c = args.config as IConfig;
        const err = mockValidate(c);
        if (err) throw err;
        this.config = clone(c);
        this.log('config', 'Настройки сохранены.');
        this.recompute();
        this.emit();
        return clone(this.config);
      }
      case 'set_monitoring': {
        this.config.monitoring = Boolean(args.on);
        this.log('info', this.config.monitoring ? 'Наблюдение включено.' : 'Наблюдение выключено.');
        this.recompute();
        this.emit();
        return this.state();
      }
      case 'test_targets':
        return clone(
          [
            ...this.agents,
            ...MOCK_HELPERS.filter(h => this.agents.some(a => a.pid === h.rootPid)),
          ].filter(a => this.config.targets[a.target])
        );
      case 'recent_events':
        return this.events.slice(0, Number(args.limit ?? 200));
      default:
        throw `Неизвестная команда: ${cmd}`;
    }
  }
}
