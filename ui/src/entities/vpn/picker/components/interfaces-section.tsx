import { Spin, Tag, Typography } from 'antd';

import type { InterfaceWatchItem, WatchItem } from 'common/types';
import { interfaceItem, watchKey } from '../../watch/watch-items';
import { OwnerLabel } from '../../owner';
import type { IDiscoveredInterface } from 'common/types';
import { PickerRow } from './picker-row';
import { PickerSection } from './picker-section';
import { INTERFACES_SECTION_TEXTS as T } from './interfaces-section.consts';

interface IInterfacesSectionProps {
  loaded: boolean;
  ifaces: IDiscoveredInterface[];
  missing: InterfaceWatchItem[];
  selected: Set<string>;
  showAll: boolean;
  onShowAll: (v: boolean) => void;
  toggle: (item: WatchItem, on: boolean) => void;
}

/** Туннельные интерфейсы (или все) и выбранные, которых сейчас нет. */
export function InterfacesSection({
  loaded,
  ifaces,
  missing,
  selected,
  showAll,
  onShowAll,
  toggle,
}: IInterfacesSectionProps) {
  return (
    <PickerSection
      title={T.title}
      showAll={{
        label: T.showAll,
        ariaLabel: T.showAllLabel,
        checked: showAll,
        onChange: onShowAll,
      }}
    >
      {!loaded && <Spin size="small" />}
      {loaded && ifaces.length === 0 && missing.length === 0 && (
        <Typography.Text type="secondary">{T.empty}</Typography.Text>
      )}
      {ifaces.map(i => {
        const item = interfaceItem(i);
        return (
          <PickerRow
            key={i.name}
            checked={selected.has(watchKey(item))}
            onChange={on => toggle(item, on)}
            title={
              <>
                {i.name}
                <OwnerLabel owner={i.owner} /> {i.tunnel && <Tag color="blue">{T.tunnel}</Tag>}
                {!i.up && <Tag>{T.down}</Tag>}
                {i.desc && <Typography.Text type="secondary"> {i.desc}</Typography.Text>}
              </>
            }
            sub={i.addrs.length ? i.addrs.join(', ') : T.noAddr}
          />
        );
      })}
      {missing.map(w => (
        <PickerRow
          key={watchKey(w)}
          checked
          onChange={() => toggle(w, false)}
          title={
            <>
              {w.name} <Tag>{T.missing}</Tag>
            </>
          }
          sub={w.addr ? w.addr : ''}
        />
      ))}
    </PickerSection>
  );
}
