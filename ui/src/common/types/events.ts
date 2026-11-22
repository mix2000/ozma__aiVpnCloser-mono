// Контракт ядра: журнал событий (recent_events).

/* Запись журнала. */
export interface IEventRecord {
  /* RFC 3339. */
  at: string;
  kind: string;
  message: string;
}
