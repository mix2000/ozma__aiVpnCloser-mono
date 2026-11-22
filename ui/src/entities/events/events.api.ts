import { apiService } from 'common/services/api';

import type { IEventRecord } from 'common/types';

export const eventsApi = {
  /** Последние события, новые первыми. */
  recentEvents: (limit: number) => apiService.command<IEventRecord[]>('recent_events', { limit }),
};
