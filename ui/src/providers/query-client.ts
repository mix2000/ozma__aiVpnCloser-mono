import { QueryClient } from '@tanstack/react-query';

/**
 * Клиент React Query. Бэкенд локальный (команды Tauri): повторы и
 * перезапросы по фокусу выключены по умолчанию — каждый запрос включает
 * то, что ему нужно, сам.
 */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false, refetchOnReconnect: false },
      mutations: { retry: false },
    },
  });
}

export const queryClient = createQueryClient();
