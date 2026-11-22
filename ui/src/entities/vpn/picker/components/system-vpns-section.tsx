import { Tag, Typography } from 'antd';

import type { WatchItem } from 'common/types';
import { watchKey } from '../../watch/watch-items';
import type { IDiscoveredSystemVpn } from 'common/types';
import { PickerRow } from './picker-row';
import { PickerSection } from './picker-section';
import { SYSTEM_VPNS_SECTION_TEXTS as T } from './system-vpns-section.consts';

interface ISystemVpnsSectionProps {
  vpns: IDiscoveredSystemVpn[] | null;
  selected: Set<string>;
  toggle: (item: WatchItem, on: boolean) => void;
}

/** Системные VPN-сервисы (macOS scutil / Windows RAS). */
export function SystemVpnsSection({ vpns, selected, toggle }: ISystemVpnsSectionProps) {
  return (
    <PickerSection title={T.title}>
      {vpns && vpns.length === 0 && <Typography.Text type="secondary">{T.empty}</Typography.Text>}
      {(vpns ?? []).map(v => {
        const item: WatchItem = { kind: 'system_vpn', id: v.id, name: v.name };
        return (
          <PickerRow
            key={v.id}
            checked={selected.has(watchKey(item))}
            onChange={on => toggle(item, on)}
            title={
              <>
                {v.name}{' '}
                {v.connected ? <Tag color="green">{T.connected}</Tag> : <Tag>{T.disconnected}</Tag>}
              </>
            }
          />
        );
      })}
    </PickerSection>
  );
}
