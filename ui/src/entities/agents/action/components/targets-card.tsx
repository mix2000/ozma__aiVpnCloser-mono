import { Button, Card, Checkbox, Space, Typography } from 'antd';
import { useState } from 'react';

import { TARGET_KINDS, TARGET_LABELS } from '../../agents.consts';
import { countByTarget, isRoot, sessionsText, targetCountText } from '../agent-counts';
import type { IAgent, TargetKind, Targets } from 'common/types';
import { SessionsList } from './sessions-list';
import { TARGETS_CARD_TEXTS as T } from './targets-card.consts';
import styles from './targets-card.module.scss';

interface ITargetsCardProps {
  targets: Targets;
  /** Кого затронет закрытие сейчас (с потомками). */
  affected: IAgent[];
  onToggle: (target: TargetKind, on: boolean) => void;
}

/** Цели-чекбоксы с числом найденных процессов и список сессий. */
export function TargetsCard({ targets, affected, onToggle }: ITargetsCardProps) {
  const [showList, setShowList] = useState(false);
  const counts = countByTarget(affected);
  const sessions = affected.filter(isRoot).length;

  return (
    <Card
      size="small"
      title={T.title}
      extra={
        <Button size="small" onClick={() => setShowList(v => !v)}>
          {showList ? T.hideList : T.showList(sessionsText(sessions))}
        </Button>
      }
    >
      <Space orientation="vertical" size={6} className={styles.targets}>
        {TARGET_KINDS.map(t => (
          <Checkbox key={t} checked={targets[t]} onChange={e => onToggle(t, e.target.checked)}>
            {TARGET_LABELS[t]}{' '}
            <Typography.Text type="secondary">
              {targets[t] ? `— ${targetCountText(counts[t])}` : ''}
            </Typography.Text>
          </Checkbox>
        ))}
      </Space>
      {showList && (
        <div className={styles.list}>
          <SessionsList agents={affected} />
        </div>
      )}
    </Card>
  );
}
