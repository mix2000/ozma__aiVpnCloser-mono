// Контракт ядра: настройки (get_config / save_config) и их пределы (get_limits).
import type { Targets } from './agents';
import type { WatchItem, WatchMode } from './vpn';

/* Настройки. */
export interface IConfig {
  watch: WatchItem[];
  mode: WatchMode;
  hosts: string[];
  targets: Targets;
  freezeImmediately: boolean;
  closeAfterSec: number;
  killGraceSec: number;
  blockWhileDown: boolean;
  pollMs: number;
  confirmFailures: number;
  dryRun: boolean;
  notify: boolean;
  autostart: boolean;
  monitoring: boolean;
}

/* Частичное сохранение: что поменять в настройках. */
export type ConfigPatch = Partial<IConfig>;

/* Сохранить изменения; false — бэкенд отказал (сообщение уже показано). */
export type SaveConfig = (patch: ConfigPatch) => Promise<boolean>;

/* Допустимый диапазон, включительно. */
export interface IRange {
  min: number;
  max: number;
}

/* Пределы числовых настроек — единственный источник в ядре. */
export interface ILimits {
  pollMs: IRange;
  confirmFailures: IRange;
  closeAfterSec: IRange;
  killGraceSec: IRange;
}
