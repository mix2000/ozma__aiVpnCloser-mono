import { ReloadOutlined } from '@ant-design/icons';
import { Button, Space, Typography } from 'antd';

import { WATCH_PICKER_TEXTS as T } from '../watch-picker.consts';
import styles from '../watch-picker.module.scss';

interface IPickerToolbarProps {
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
  /* Нет найденного — кнопки «рекомендуемое» нет. */
  onSuggest: (() => void) | null;
}

/** «Обновить», «Выбрать рекомендуемое», ошибка загрузки. */
export function PickerToolbar({ loading, error, onRefresh, onSuggest }: IPickerToolbarProps) {
  return (
    <Space className={styles.toolbar} wrap>
      <Button icon={<ReloadOutlined />} onClick={onRefresh} loading={loading}>
        {T.refresh}
      </Button>
      {onSuggest && <Button onClick={onSuggest}>{T.suggest}</Button>}
      {error && <Typography.Text type="danger">{error}</Typography.Text>}
    </Space>
  );
}
