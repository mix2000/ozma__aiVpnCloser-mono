import { Card, Switch } from 'antd';

import { SettingRow } from 'common/components';
import type { IConfig, IRange, SaveConfig } from 'common/types';

import { CloseAfterField } from './close-after-field';
import { CLOSE_SETTINGS_TEXTS as T } from './close-settings-card.consts';

interface ICloseSettingsCardProps {
  config: IConfig;
  /* Пределы «Закрыть через» — из ядра. */
  closeAfter: IRange;
  save: SaveConfig;
}

/** «Как закрывать»: заморозка, задержка, блокировка новых, «только смотреть». */
export function CloseSettingsCard({ config, closeAfter, save }: ICloseSettingsCardProps) {
  return (
    <Card size="small" title={T.title}>
      <SettingRow title={T.freeze} hint={T.freezeHint}>
        <Switch
          checked={config.freezeImmediately}
          onChange={v => void save({ freezeImmediately: v })}
        />
      </SettingRow>

      <CloseAfterField
        value={config.closeAfterSec}
        range={closeAfter}
        onCommit={closeAfterSec => void save({ closeAfterSec })}
      />

      <SettingRow title={T.block} hint={T.blockHint}>
        <Switch checked={config.blockWhileDown} onChange={v => void save({ blockWhileDown: v })} />
      </SettingRow>

      <SettingRow title={T.dryRun} hint={T.dryRunHint} last>
        <Switch checked={config.dryRun} onChange={v => void save({ dryRun: v })} />
      </SettingRow>
    </Card>
  );
}
