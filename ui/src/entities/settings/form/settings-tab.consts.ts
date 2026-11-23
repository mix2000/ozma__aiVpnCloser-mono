export const SETTINGS_TEXTS = {
  autostart: 'Запускать при входе в систему',
  autostartHint: 'Программа стартует сама и сразу следит за VPN.',
  notify: 'Уведомления',
  notifyHint: 'Сообщать, когда VPN пропал, вернулся и когда агенты закрыты.',
  poll: 'Как часто проверять VPN',
  pollHint: 'Чем чаще, тем быстрее замечу, что VPN пропал.',
  pollLabel: 'Интервал проверки, мс',
  confirm: 'Сколько неудачных проверок подряд считать «VPN пропал»',
  confirmHint: 'Защита от случайных сбоев: одна неудачная проверка ещё не повод.',
  confirmLabel: 'Подтверждений',
  grace: 'Сколько ждать после «закрыть», прежде чем закрыть принудительно',
  graceHint: 'Агенту даётся время завершиться самому. На Windows не действует: закрывается сразу.',
  graceLabel: 'Ожидание перед принудительным закрытием',
  trayNoteStart: 'Закрыть окно — программа останется в ',
  trayNoteEnd: '. Выход — из её меню.',
  trayMac: 'меню-баре',
  trayOther: 'трее',
};

/** Шаг и единица полей; пределы — из ядра (get_limits). */
export const POLL_MS = { step: 100, unit: 'мс' };
export const CONFIRM_FAILURES = { step: 1, unit: 'раз' };
export const KILL_GRACE_SEC = { step: 1, unit: 'с' };
