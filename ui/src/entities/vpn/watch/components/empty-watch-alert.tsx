import { Alert, Button } from 'antd';

import { EMPTY_WATCH_TEXTS as T } from './empty-watch-alert.consts';

/** Первый запуск: ничего не выбрано — предложить выбрать. */
export function EmptyWatchAlert({ onChoose }: { onChoose: () => void }) {
  return (
    <Alert
      type="info"
      showIcon
      title={T.title}
      description={T.description}
      action={
        <Button type="primary" onClick={onChoose}>
          {T.choose}
        </Button>
      }
    />
  );
}
