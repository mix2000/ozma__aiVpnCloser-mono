import { apiService } from 'common/services/api';

import type { IAgent } from 'common/types';

export const agentsApi = {
  /** Все процессы включённых целей вместе с потомками — без действий. */
  testTargets: () => apiService.command<IAgent[]>('test_targets'),
};
