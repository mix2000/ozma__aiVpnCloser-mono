import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { errorText, queryKeys } from 'common/services/api';
import type { IAppState } from 'common/types';

import { eventsApi } from '../events.api';
import { EVENTS_LIMIT } from './journal.consts';

/**
 * Журнал: читается, пока вкладка открыта; перечитывается, когда меняется
 * статус монитора, и по кнопке.
 */
export function useRecentEvents(
  active: boolean,
  state: Pick<IAppState, 'status' | 'lastChangeAt' | 'monitoring'>
) {
  const client = useQueryClient();
  const q = useQuery({
    queryKey: queryKeys.events,
    queryFn: () => eventsApi.recentEvents(EVENTS_LIMIT),
    enabled: active,
  });
  const { status, lastChangeAt, monitoring } = state;

  useEffect(() => {
    void client.invalidateQueries({ queryKey: queryKeys.events });
  }, [client, status, lastChangeAt, monitoring]);

  return {
    events: q.data ?? null,
    error: q.error ? errorText(q.error) : null,
    load: () => void q.refetch(),
  };
}
