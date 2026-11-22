import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { App } from 'antd';
import { useEffect } from 'react';

import { errorText, queryKeys } from 'common/services/api';
import type { IAppState } from 'common/types';

import { monitorApi } from '../monitor.api';

/**
 * Состояние монитора: get_state + пуш "state-changed" в кеш. Пуш важнее:
 * ответ get_state, пришедший после пуша, его не перетирает.
 */
export function useAppState() {
  const client = useQueryClient();

  const q = useQuery({
    queryKey: queryKeys.state,
    queryFn: async () => {
      const s = await monitorApi.getState();
      return client.getQueryData<IAppState>(queryKeys.state) ?? s;
    },
    staleTime: Infinity,
  });

  useEffect(
    () => monitorApi.onStateChanged(s => client.setQueryData(queryKeys.state, s)),
    [client]
  );

  return { state: q.data ?? null, error: q.error ? errorText(q.error) : null };
}

/** Переключатель «Следить»: сохраняет и сразу кладёт новое состояние в кеш. */
export function useMonitoringToggle() {
  const { message } = App.useApp();
  const client = useQueryClient();
  const m = useMutation({
    mutationFn: monitorApi.setMonitoring,
    onSuccess: s => client.setQueryData(queryKeys.state, s),
    onError: e => message.error(errorText(e)),
  });
  return { switching: m.isPending, toggle: (on: boolean) => m.mutate(on) };
}
