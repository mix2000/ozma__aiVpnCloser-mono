import { useQuery } from '@tanstack/react-query';

import { errorText, queryKeys } from 'common/services/api';

import { DISCOVERY_REFRESH_MS } from '../vpn.consts';
import { vpnApi } from '../vpn.api';

/**
 * Найденное на машине: сразу и раз в несколько секунд, плюс «Обновить».
 * Кеш не переживает закрытие окна выбора — при следующем открытии всё заново.
 */
export function useDiscovery() {
  const q = useQuery({
    queryKey: queryKeys.discovery,
    queryFn: vpnApi.discover,
    refetchInterval: DISCOVERY_REFRESH_MS,
    gcTime: 0,
  });
  return {
    found: q.data ?? null,
    loadError: q.error ? errorText(q.error) : null,
    loading: q.isFetching,
    refresh: () => q.refetch(),
  };
}
