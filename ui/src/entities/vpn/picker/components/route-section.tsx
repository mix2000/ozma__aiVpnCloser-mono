import { Button, Input, Space, Typography } from 'antd';

import { PickerRow } from './picker-row';
import { PickerSection } from './picker-section';
import { ROUTE_SECTION_TEXTS as T } from './route-section.consts';
import styles from './route-section.module.scss';

interface IRouteSectionProps {
  on: boolean;
  onToggle: (on: boolean) => void;
  hostsText: string;
  hostsCount: number;
  onHostsText: (text: string) => void;
  onResetHosts: () => void;
}

/** Пункт «трафик к агентам идёт через VPN» и редактор хостов. */
export function RouteSection({
  on,
  onToggle,
  hostsText,
  hostsCount,
  onHostsText,
  onResetHosts,
}: IRouteSectionProps) {
  return (
    <PickerSection title={T.section}>
      <PickerRow checked={on} onChange={onToggle} title={T.title} sub={T.hint} />
      {on && (
        <div className={styles.hosts}>
          <Input.TextArea
            value={hostsText}
            onChange={e => onHostsText(e.target.value)}
            autoSize={{ minRows: 3, maxRows: 8 }}
            placeholder={T.hostsPlaceholder}
            aria-label={T.hostsLabel}
          />
          <Space className={styles.hostsFooter}>
            <Typography.Text type="secondary" className={styles.hostsCount}>
              {T.hostsCount(hostsCount)}
            </Typography.Text>
            <Button size="small" type="link" onClick={onResetHosts}>
              {T.resetHosts}
            </Button>
          </Space>
        </div>
      )}
    </PickerSection>
  );
}
