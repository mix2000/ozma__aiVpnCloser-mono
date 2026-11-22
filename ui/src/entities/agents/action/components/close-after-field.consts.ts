export const CLOSE_AFTER_STEP_SEC = 5;

export const CLOSE_AFTER_TEXTS = {
  label: 'Закрыть через',
  inputLabel: 'Закрыть через, секунд',
  unit: 'с',
  tooltip: (v?: number) => `${v} с`,
  now: 'Закрою сразу, без ожидания.',
  later: (n: number) => `Если VPN не вернётся за ${n} с — закрою агентов.`,
};
