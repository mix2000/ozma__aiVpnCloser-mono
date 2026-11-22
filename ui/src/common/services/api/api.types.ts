/* Аргументы команды Tauri (camelCase, как в контракте). */
export type ApiArgs = Record<string, unknown>;

/* Подставной бэкенд: те же команды и события, что у Tauri (мок для браузера и тестов). */
export interface IFallbackBackend {
  call(cmd: string, args?: ApiArgs): Promise<unknown>;
  listen<T>(event: string, cb: (payload: T) => void): () => void;
}

/* Отписка от события. */
export type Unsubscribe = () => void;
