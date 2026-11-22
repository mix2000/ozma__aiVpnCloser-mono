import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { queryKeys } from 'common/services/api';
import type { IAgent, Targets } from 'common/types';

import { agentsApi } from '../agents.api';
import { TARGETS_REFRESH_MS } from '../agents.consts';

/** Кого затронет закрытие сейчас: сразу, раз в несколько секунд и при смене целей. */
export function useTestTargets(targets: Targets): IAgent[] {
  const client = useQueryClient();
  const q = useQuery({
    queryKey: queryKeys.targets,
    queryFn: agentsApi.testTargets,
    refetchInterval: TARGETS_REFRESH_MS,
  });

  useEffect(() => {
    void client.invalidateQueries({ queryKey: queryKeys.targets });
  }, [client, targets]);

  return q.data ?? [];
}
