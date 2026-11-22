import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { App } from 'antd';
import { useCallback, useEffect } from 'react';

import { errorText, queryKeys } from 'common/services/api';
import type { ConfigPatch, IConfig } from 'common/types';

import { settingsApi } from '../settings.api';

/**
 * Конфиг с сохранением на каждое изменение (оптимистично): при ошибке
 * проверки — сообщение и откат. Перечитывается, когда окно снова в фокусе
 * (его могли открыть из трея после «Следить» в меню).
 */
export function useConfig() {
  const { message } = App.useApp();
  const client = useQueryClient();
  const q = useQuery({
    queryKey: queryKeys.config,
    queryFn: settingsApi.getConfig,
    refetchOnWindowFocus: 'always',
  });

  useEffect(() => {
    if (q.error) message.error(errorText(q.error));
  }, [q.error, message]);

  const m = useMutation({
    mutationFn: (patch: ConfigPatch) => {
      const base = client.getQueryData<IConfig>(queryKeys.config) as IConfig;
      return settingsApi.saveConfig({ ...base, ...patch });
    },
    onMutate: async patch => {
      await client.cancelQueries({ queryKey: queryKeys.config });
      const previous = client.getQueryData<IConfig>(queryKeys.config);
      if (previous) client.setQueryData(queryKeys.config, { ...previous, ...patch });
      return { previous };
    },
    onError: (e, _patch, ctx) => {
      message.error(errorText(e));
      if (ctx?.previous) client.setQueryData(queryKeys.config, ctx.previous);
    },
    onSuccess: saved => client.setQueryData(queryKeys.config, saved),
  });

  const { mutateAsync } = m;
  const save = useCallback(
    async (patch: ConfigPatch): Promise<boolean> => {
      if (!client.getQueryData(queryKeys.config)) return false;
      try {
        await mutateAsync(patch);
        return true;
      } catch {
        return false;
      }
    },
    [client, mutateAsync]
  );

  /** Монитор сам меняет monitoring — держим конфиг в курсе. */
  const syncMonitoring = useCallback(
    (monitoring: boolean, dryRun: boolean) => {
      const c = client.getQueryData<IConfig>(queryKeys.config);
      if (!c || (c.monitoring === monitoring && c.dryRun === dryRun)) return;
      client.setQueryData(queryKeys.config, { ...c, monitoring, dryRun });
    },
    [client]
  );

  return { config: q.data ?? null, save, syncMonitoring };
}

/** Пределы числовых настроек — из ядра (get_limits). */
export function useLimits() {
  const q = useQuery({
    queryKey: queryKeys.limits,
    queryFn: settingsApi.getLimits,
    staleTime: Infinity,
  });
  return q.data ?? null;
}
