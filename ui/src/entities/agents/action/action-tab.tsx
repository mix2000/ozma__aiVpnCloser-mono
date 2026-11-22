import { Alert, Space } from 'antd';

import type { IAppState, IConfig, ILimits, SaveConfig } from 'common/types';

import { ACTION_TAB_TEXTS as T } from './action-tab.consts';
import { affectedAgents, hasConsoleTarget } from './agent-counts';
import { CloseSettingsCard, TargetsCard } from './components';
import { useTestTargets } from './hooks';
import styles from './action-tab.module.scss';

interface IActionTabProps {
  state: IAppState;
  config: IConfig;
  limits: ILimits;
  save: SaveConfig;
}

/** Вкладка «Действие»: кого и как закрывать, когда VPN пропал. */
export function ActionTab({ state, config, limits, save }: IActionTabProps) {
  const found = useTestTargets(config.targets);
  const affected = affectedAgents(found, state.agents, config.targets);

  return (
    <Space orientation="vertical" size={12} className={styles.tab}>
      <TargetsCard
        targets={config.targets}
        affected={affected}
        onToggle={(t, on) => void save({ targets: { ...config.targets, [t]: on } })}
      />
      <CloseSettingsCard config={config} closeAfter={limits.closeAfterSec} save={save} />
      {!config.dryRun && !hasConsoleTarget(config.targets) && (
        <Alert type="warning" showIcon title={T.noConsoleTargets} />
      )}
    </Space>
  );
}
