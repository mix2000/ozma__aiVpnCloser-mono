import { Tag, Typography } from 'antd';

import { Rows, rowStyles } from 'common/components';

import { TARGET_LABELS } from '../../agents.consts';
import { helpersBySession, helpersText, isRoot } from '../agent-counts';
import type { IAgent } from 'common/types';
import { SESSIONS_LIST_TEXTS as T } from './sessions-list.consts';

/** Сессии, которых затронет закрытие, с числом вспомогательных процессов. */
export function SessionsList({ agents }: { agents: IAgent[] }) {
  const helpers = helpersBySession(agents);
  return (
    <Rows
      bordered
      items={agents.filter(isRoot)}
      rowKey={a => a.pid}
      empty={T.empty}
      render={a => (
        <>
          <span className={`${rowStyles.main} vc-selectable`}>
            {TARGET_LABELS[a.target]} · {a.name} · pid {a.pid}
            {helpers.get(a.pid) ? (
              <Typography.Text type="secondary">
                {' '}
                · {helpersText(helpers.get(a.pid)!)}
              </Typography.Text>
            ) : null}
          </span>
          {a.frozen && <Tag color="gold">{T.frozen}</Tag>}
        </>
      )}
    />
  );
}
