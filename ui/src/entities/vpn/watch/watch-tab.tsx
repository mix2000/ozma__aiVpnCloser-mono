import { Space } from 'antd';
import { useState } from 'react';

import type { IAppState } from 'common/types';
import type { IConfig, SaveConfig } from 'common/types';

import { WatchPicker } from '../picker/watch-picker';
import { EmptyWatchAlert, WatchChecksCard, WatchModeCard } from './components';
import { watchRows } from './watch-rows';
import styles from './watch-tab.module.scss';

interface IWatchTabProps {
  state: IAppState;
  config: IConfig;
  save: SaveConfig;
}

/** Вкладка «Наблюдение»: что выбрано, как проверки, правило all/any. */
export function WatchTab({ state, config, save }: IWatchTabProps) {
  const [picking, setPicking] = useState(false);
  const open = () => setPicking(true);

  return (
    <Space orientation="vertical" size={12} className={styles.tab}>
      {config.watch.length === 0 && <EmptyWatchAlert onChoose={open} />}
      <WatchChecksCard
        rows={watchRows(config.watch, state.checks, state.monitoring)}
        onChoose={open}
      />
      <WatchModeCard mode={config.mode} onChange={mode => void save({ mode })} />
      {picking && (
        <WatchPicker
          config={config}
          onCancel={() => setPicking(false)}
          onSave={async (watch, hosts) => {
            if (await save({ watch, hosts })) setPicking(false);
          }}
        />
      )}
    </Space>
  );
}
