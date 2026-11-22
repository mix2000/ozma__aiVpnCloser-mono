import { Button, Card, Empty } from 'antd';

import { Rows, rowStyles } from 'common/components';

import { OwnerLabel } from '../../owner';
import type { ICheck } from 'common/types';
import { watchKey } from '../watch-items';
import { CheckIcon } from './check-icon';
import { WATCH_CHECKS_TEXTS as T } from './watch-checks-card.consts';

interface IWatchChecksCardProps {
  rows: ICheck[];
  onChoose: () => void;
}

/** «За чем слежу»: выбранные пункты с ✓/✗ и пояснением. */
export function WatchChecksCard({ rows, onChoose }: IWatchChecksCardProps) {
  return (
    <Card
      size="small"
      title={T.title}
      extra={
        <Button size="small" onClick={onChoose}>
          {T.choose}
        </Button>
      }
    >
      {rows.length === 0 ? (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={T.empty} />
      ) : (
        <Rows
          items={rows}
          rowKey={c => watchKey(c.item)}
          render={c => (
            <>
              <CheckIcon ok={c.ok} />
              <div className={rowStyles.main}>
                <div>
                  {c.label}
                  <OwnerLabel owner={c.owner} />
                </div>
                <div className={`${rowStyles.sub} vc-selectable`}>{c.detail}</div>
              </div>
            </>
          )}
        />
      )}
    </Card>
  );
}
