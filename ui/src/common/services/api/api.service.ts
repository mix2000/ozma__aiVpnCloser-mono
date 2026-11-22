// Обёртка над транспортом для слоя сущностей: команда → промис с типом,
// событие → подписка. Сущности ходят к бэкенду только через неё.
import { call, listen } from './api.instance';
import type { ApiArgs, Unsubscribe } from './api.types';

export const apiService = {
  /* Вызвать команду Tauri; ошибка — Error с текстом от бэкенда по-русски. */
  command: <T>(name: string, args?: ApiArgs): Promise<T> => call<T>(name, args),
  /* Подписаться на событие бэкенда. */
  subscribe: <T>(event: string, cb: (payload: T) => void): Unsubscribe => listen<T>(event, cb),
};
