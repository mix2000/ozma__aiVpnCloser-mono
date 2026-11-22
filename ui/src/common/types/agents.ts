// Контракт ядра: цели и процессы агентов.

/* Вид агента. */
export type TargetKind = 'claudeCode' | 'claudeDesktop' | 'codexCli' | 'codexDesktop' | 'grok';

/* Какие цели включены. */
export type Targets = Record<TargetKind, boolean>;

/* Процесс агента (сессия или её потомок). */
export interface IAgent {
  target: TargetKind;
  pid: number;
  name: string;
  frozen: boolean;
  /* true — сама сессия агента; false — её потомок (MCP-сервер, команда агента). */
  root?: boolean;
  /* pid сессии, к которой относится процесс. */
  rootPid?: number;
}
