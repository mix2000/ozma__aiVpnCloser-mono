import { Tabs } from 'antd';

import type { IAppState, IConfig, ILimits, SaveConfig } from 'common/types';
import { ActionTab } from 'entities/agents';
import { JournalTab } from 'entities/events';
import { SettingsTab } from 'entities/settings';
import { WatchTab } from 'entities/vpn';

import { TAB_LABELS, type TabKey } from './main-page.consts';
import styles from './main-page.module.scss';

interface IMainTabsProps {
  tab: TabKey;
  onTab: (tab: TabKey) => void;
  state: IAppState;
  config: IConfig;
  limits: ILimits;
  save: SaveConfig;
}

/** Вкладки: «Наблюдение», «Действие», «Журнал», «Настройки». */
export function MainTabs({ tab, onTab, state, config, limits, save }: IMainTabsProps) {
  const items = [
    {
      key: 'watch',
      label: TAB_LABELS.watch,
      children: <WatchTab state={state} config={config} save={save} />,
    },
    {
      key: 'action',
      label: TAB_LABELS.action,
      children: <ActionTab state={state} config={config} limits={limits} save={save} />,
    },
    {
      key: 'journal',
      label: TAB_LABELS.journal,
      children: <JournalTab state={state} active={tab === 'journal'} />,
    },
    {
      key: 'settings',
      label: TAB_LABELS.settings,
      children: <SettingsTab config={config} limits={limits} save={save} />,
    },
  ];
  return (
    <Tabs
      className={styles.tabs}
      activeKey={tab}
      onChange={k => onTab(k as TabKey)}
      items={items}
    />
  );
}
