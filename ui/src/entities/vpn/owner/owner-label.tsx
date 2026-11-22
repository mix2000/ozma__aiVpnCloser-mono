import { Typography } from 'antd';

import type { IIfOwner } from 'common/types';
import { OWNER_TEXTS as T } from './owner.consts';
import { ownerText } from './owner-text';

interface IOwnerLabelProps {
  owner?: IIfOwner | null;
}

/** « (sing-box)» после имени интерфейса; «вероятно …» — серым. Нет владельца — ничего. */
export function OwnerLabel({ owner }: IOwnerLabelProps) {
  if (!owner) return null;
  const title = owner.pid ? T.pidTitle(owner.pid) : undefined;
  if (owner.confidence === 'likely') {
    return (
      <Typography.Text type="secondary" title={title}>
        {' '}
        {ownerText(owner)}
      </Typography.Text>
    );
  }
  return <span title={title}> {ownerText(owner)}</span>;
}
