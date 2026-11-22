import {
  CheckOutlined,
  ClockCircleOutlined,
  CloseOutlined,
  PauseOutlined,
} from '@ant-design/icons';
import { Switch, Tag, Tooltip } from 'antd';
import type { ReactNode } from 'react';

import type { IAppState, StatusColor } from 'common/types';
import styles from './status-header.module.scss';
import { statusHint } from './status-hint';
import { STATUS_HEADER_TEXTS as T } from './status.consts';

interface IStatusHeaderProps {
  state: IAppState;
  canMonitor: boolean;
  switching: boolean;
  onToggle: (on: boolean) => void;
  onChooseWatch: () => void;
}

const ICONS: Record<StatusColor, ReactNode> = {
  green: <CheckOutlined />,
  yellow: <ClockCircleOutlined />,
  red: <CloseOutlined />,
  gray: <PauseOutlined />,
};

/** Шапка: большой статус цветом, переключатель «Следить», бейдж «Только смотреть». */
export function StatusHeader({
  state,
  canMonitor,
  switching,
  onToggle,
  onChooseWatch,
}: IStatusHeaderProps) {
  const { color } = state;
  const sw = (
    <Switch
      checked={state.monitoring}
      loading={switching}
      disabled={!canMonitor && !state.monitoring}
      onChange={onToggle}
      aria-label={T.watch}
    />
  );
  return (
    <div className={`${styles.card} ${styles[color]}`} data-status={color}>
      <div className={styles.icon}>{ICONS[color]}</div>
      <div className={styles.text}>
        <div className={styles.title}>{state.title}</div>
        <div className={styles.hint}>
          {!canMonitor ? (
            <>
              {T.chooseFirst}{' '}
              <a onClick={onChooseWatch} role="button">
                {T.chooseLink}
              </a>
              .
            </>
          ) : (
            statusHint(state)
          )}
        </div>
      </div>
      <div className={styles.side}>
        <label className={styles.switch}>
          {T.watch}
          {canMonitor || state.monitoring ? sw : <Tooltip title={T.chooseTooltip}>{sw}</Tooltip>}
        </label>
        {state.dryRun && (
          <Tooltip title={T.dryRunTooltip}>
            <Tag color="blue" className={styles.tag}>
              {T.dryRun}
            </Tag>
          </Tooltip>
        )}
      </div>
    </div>
  );
}
