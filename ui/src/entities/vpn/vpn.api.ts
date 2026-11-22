import { apiService } from 'common/services/api';

import type { IDiscovery } from 'common/types';

export const vpnApi = {
  discover: () => apiService.command<IDiscovery>('discover'),
};
