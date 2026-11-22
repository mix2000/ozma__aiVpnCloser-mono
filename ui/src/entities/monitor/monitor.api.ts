import { apiService } from 'common/services/api';

import type { IAppState } from 'common/types';

/** Событие бэкенда при каждом изменении состояния. */
export const STATE_EVENT = 'state-changed';

export const monitorApi = {
  getState: () => apiService.command<IAppState>('get_state'),
  setMonitoring: (on: boolean) => apiService.command<IAppState>('set_monitoring', { on }),
  /* Пуш состояния из бэкенда. Возвращает функцию отписки. */
  onStateChanged: (cb: (s: IAppState) => void) => apiService.subscribe<IAppState>(STATE_EVENT, cb),
};
