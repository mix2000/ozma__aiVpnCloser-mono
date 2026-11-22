import { Card, Radio, Typography } from 'antd';

import type { WatchMode } from 'common/types';
import { WATCH_MODE_OPTIONS, WATCH_MODE_TEXTS as T } from './watch-mode-card.consts';
import styles from './watch-mode-card.module.scss';

interface IWatchModeCardProps {
  mode: WatchMode;
  onChange: (mode: WatchMode) => void;
}

/** Правило объединения: все пункты или хотя бы один. */
export function WatchModeCard({ mode, onChange }: IWatchModeCardProps) {
  return (
    <Card size="small" title={T.title}>
      <Radio.Group
        value={mode}
        onChange={e => onChange(e.target.value)}
        options={WATCH_MODE_OPTIONS}
      />
      <Typography.Paragraph type="secondary" className={styles.hint}>
        {T.hint}
      </Typography.Paragraph>
    </Card>
  );
}
