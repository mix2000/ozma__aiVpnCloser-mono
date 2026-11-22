import { ReloadOutlined } from '@ant-design/icons';
import { Button, Card, Empty, Spin, Typography } from 'antd';

import { Rows } from 'common/components';
import type { IAppState } from 'common/types';

import { EventRow } from './components';
import { eventKey } from './event-tag';
import { useRecentEvents } from './hooks';
import { JOURNAL_TEXTS as T } from './journal.consts';

interface IJournalTabProps {
  state: IAppState;
  active: boolean;
}

/** Вкладка «Журнал»: перечитывается при открытии и при смене статуса. */
export function JournalTab({ state, active }: IJournalTabProps) {
  const { events, error, load } = useRecentEvents(active, state);

  return (
    <Card
      size="small"
      title={T.title}
      extra={
        <Button size="small" icon={<ReloadOutlined />} onClick={load}>
          {T.refresh}
        </Button>
      }
    >
      {error && <Typography.Text type="danger">{error}</Typography.Text>}
      {!events && !error && <Spin size="small" />}
      {events && (
        <Rows
          items={events}
          rowKey={eventKey}
          empty={<Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={T.empty} />}
          render={e => <EventRow event={e} />}
        />
      )}
    </Card>
  );
}
