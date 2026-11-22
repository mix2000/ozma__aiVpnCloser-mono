/* Ключи React Query — константы: опечатка в литерале тихо ломала бы
 * обновление кеша. Обновление — через setQueryData / invalidateQueries. */
export const queryKeys = {
  state: ['state'] as const,
  config: ['config'] as const,
  limits: ['limits'] as const,
  discovery: ['discovery'] as const,
  targets: ['targets'] as const,
  events: ['events'] as const,
};
