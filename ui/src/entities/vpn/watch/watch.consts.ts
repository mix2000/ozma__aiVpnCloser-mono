/**
 * Название пункта, пока монитор не прислал проверку (дальше — `ICheck.label`
 * от ядра, той же формулировкой: models/config.rs → WatchItem::label).
 */
export const WATCH_TITLES = {
  interface: (name: string, addr?: string | null) =>
    addr ? `Интерфейс ${name} (${addr})` : `Интерфейс ${name}`,
  process: (name: string) => `Программа ${name}`,
  systemVpn: (name: string) => `Системный VPN «${name}»`,
  route: 'Трафик к агентам идёт через VPN',
};

/** Пояснение к пункту, по которому монитор ещё не прислал проверку. */
export const CHECK_PENDING_TEXT = 'проверяю…';
export const CHECK_PAUSED_TEXT = 'проверка начнётся, когда включите «Следить»';
