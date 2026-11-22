import { apiService } from 'common/services/api';
import type { IConfig, ILimits } from 'common/types';

export const settingsApi = {
  getConfig: () => apiService.command<IConfig>('get_config'),
  getLimits: () => apiService.command<ILimits>('get_limits'),
  /* Проверить и сохранить; ошибка проверки — текст по-русски. */
  saveConfig: (config: IConfig) => apiService.command<IConfig>('save_config', { config }),
};
