// Контракт ядра: что считать «VPN включён», проверки и найденное на машине.

/* Пункт наблюдения (WatchItem ядра). */
export type WatchItem =
  | { kind: 'interface'; name: string; addr?: string | null }
  | { kind: 'process'; name: string }
  | { kind: 'system_vpn'; id: string; name: string }
  | { kind: 'route' };

export type InterfaceWatchItem = Extract<WatchItem, { kind: 'interface' }>;
export type ProcessWatchItem = Extract<WatchItem, { kind: 'process' }>;

/* Правило объединения: все пункты или хотя бы один. */
export type WatchMode = 'all' | 'any';

/* Насколько уверенно определена программа-владелец туннеля. */
export type OwnerConfidence = 'exact' | 'likely';

/* Программа, которой принадлежит туннельный интерфейс. */
export interface IIfOwner {
  /* Что показать: «sing-box», «OpenVPN», имя системного VPN. */
  name: string;
  pid?: number | null;
  confidence: OwnerConfidence;
}

/* Проверка одного пункта. */
export interface ICheck {
  item: WatchItem;
  /* null — пока неизвестно (например, DNS не ответил). */
  ok: boolean | null;
  /* Название пункта от ядра: «Программа xray», «Интерфейс utun233 (…)». */
  label: string;
  detail: string;
  /* Для интерфейса — программа-владелец найденного туннеля. */
  owner?: IIfOwner | null;
}

/* Сетевой интерфейс. */
export interface IDiscoveredInterface {
  name: string;
  /* Адреса строками, без link-local IPv6. */
  addrs: string[];
  up: boolean;
  tunnel: boolean;
  /* Описание адаптера (Windows). */
  desc?: string | null;
  /* Программа-владелец туннеля, если удалось определить. */
  owner?: IIfOwner | null;
}

/* Процесс, похожий (или нет) на VPN-клиент. */
export interface IDiscoveredProcess {
  name: string;
  pid: number;
  knownVpn: boolean;
}

/* Системный VPN-сервис (macOS scutil / Windows RAS). */
export interface IDiscoveredSystemVpn {
  id: string;
  name: string;
  connected: boolean;
}

/* Всё найденное (discover). */
export interface IDiscovery {
  interfaces: IDiscoveredInterface[];
  processes: IDiscoveredProcess[];
  systemVpns: IDiscoveredSystemVpn[];
}
