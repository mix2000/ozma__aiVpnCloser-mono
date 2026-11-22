import { Alert, Spin } from 'antd';
import { useEffect, useState } from 'react';

import { StatusHeader, useAppState, useMonitoringToggle } from 'entities/monitor';
import { useConfig, useLimits } from 'entities/settings';

import { MAIN_PAGE_TEXTS, type TabKey } from './main-page.consts';
import styles from './main-page.module.scss';
import { MainTabs } from './main-tabs';

/** Единственная страница: шапка статуса и вкладки четырёх сущностей. */
export function MainPage() {
  const { state, error } = useAppState();
  const { config, save, syncMonitoring } = useConfig();
  const limits = useLimits();
  const { switching, toggle } = useMonitoringToggle();
  const [tab, setTab] = useState<TabKey>('watch');

  useEffect(() => {
    if (state) syncMonitoring(state.monitoring, state.dryRun);
  }, [state, syncMonitoring]);

  if (error) {
    return (
      <div className={styles.page}>
        <Alert type="error" showIcon title={MAIN_PAGE_TEXTS.noConnection} description={error} />
      </div>
    );
  }
  if (!state || !config || !limits) {
    return (
      <div className={`${styles.page} ${styles.center}`}>
        <Spin />
      </div>
    );
  }
  return (
    <div className={styles.page}>
      <StatusHeader
        state={state}
        canMonitor={config.watch.length > 0 || state.checks.length > 0}
        switching={switching}
        onToggle={toggle}
        onChooseWatch={() => setTab('watch')}
      />
      <MainTabs
        tab={tab}
        onTab={setTab}
        state={state}
        config={config}
        limits={limits}
        save={save}
      />
    </div>
  );
}
