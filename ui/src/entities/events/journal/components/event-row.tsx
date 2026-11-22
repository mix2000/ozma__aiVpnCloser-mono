import { Tag, Typography } from 'antd';

import { rowStyles } from 'common/components';
import { formatTime } from 'common/utils';

import { eventTag } from '../event-tag';
import type { IEventRecord } from 'common/types';
import styles from './event-row.module.scss';

/** Строка журнала: время, метка вида, сообщение. */
export function EventRow({ event }: { event: IEventRecord }) {
  const tag = eventTag(event.kind);
  return (
    <>
      <Typography.Text type="secondary" className={styles.time}>
        {formatTime(event.at)}
      </Typography.Text>
      {tag && (
        <Tag color={tag.color} className={styles.tag}>
          {tag.label}
        </Tag>
      )}
      <span className={`${rowStyles.main} vc-selectable`}>{event.message}</span>
    </>
  );
}
