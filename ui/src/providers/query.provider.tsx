import { QueryClientProvider, type QueryClient } from '@tanstack/react-query';
import type { ReactNode } from 'react';

import { queryClient as appQueryClient } from './query-client';

interface IQueryProviderProps {
  children: ReactNode;
  /* Свой клиент (тесты); по умолчанию — общий. */
  client?: QueryClient;
}

export function QueryProvider({ children, client = appQueryClient }: IQueryProviderProps) {
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}
