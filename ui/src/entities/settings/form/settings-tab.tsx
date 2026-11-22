import { Card, Switch, Typography } from 'antd';

import { NumberSetting, SettingRow } from 'common/components';

import type { IConfig, ILimits, SaveConfig } from 'common/types';

import {
  CONFIRM_FAILURES,
  KILL_GRACE_SEC,
  POLL_MS,
  SETTINGS_TEXTS as T,
} from './settings-tab.consts';
import styles from './settings-tab.module.scss';

interface ISettingsTabProps {
  config: IConfig;
  /* Пределы полей — из ядра (get_limits). */
  limits: ILimits;
  save: SaveConfig;
}

const isMac = () => navigator.platform.startsWith('Mac');

/** Вкладка «Настройки»: автозапуск, уведомления, интервалы. */
export function SettingsTab({ config, limits, save }: ISettingsTabProps) {
  return (
    <Card size="small">
      <SettingRow title={T.autostart} hint={T.autostartHint}>
        <Switch checked={config.autostart} onChange={v => void save({ autostart: v })} />
      </SettingRow>
      <SettingRow title={T.notify} hint={T.notifyHint}>
        <Switch checked={config.notify} onChange={v => void save({ notify: v })} />
      </SettingRow>
      <SettingRow title={T.poll} hint={T.pollHint}>
        <NumberSetting
          value={config.pollMs}
          {...POLL_MS}
          {...limits.pollMs}
          label={T.pollLabel}
          onCommit={pollMs => void save({ pollMs })}
        />
      </SettingRow>
      <SettingRow title={T.confirm} hint={T.confirmHint}>
        <NumberSetting
          value={config.confirmFailures}
          {...CONFIRM_FAILURES}
          {...limits.confirmFailures}
          label={T.confirmLabel}
          onCommit={confirmFailures => void save({ confirmFailures })}
        />
      </SettingRow>
      <SettingRow title={T.grace} hint={T.graceHint} last>
        <NumberSetting
          value={config.killGraceSec}
          {...KILL_GRACE_SEC}
          {...limits.killGraceSec}
          label={T.graceLabel}
          onCommit={killGraceSec => void save({ killGraceSec })}
        />
      </SettingRow>
      <Typography.Paragraph type="secondary" className={styles.note}>
        {T.trayNoteStart}
        {isMac() ? T.trayMac : T.trayOther}
        {T.trayNoteEnd}
      </Typography.Paragraph>
    </Card>
  );
}
