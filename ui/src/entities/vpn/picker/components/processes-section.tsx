import { Tag, Typography } from 'antd';

import type { IDiscoveredProcess, ProcessWatchItem, WatchItem } from 'common/types';

import { watchKey } from '../../watch/watch-items';
import { ManualProcess } from './manual-process';
import { PickerRow } from './picker-row';
import { PickerSection } from './picker-section';
import { PROCESSES_SECTION_TEXTS as T } from './processes-section.consts';

interface IProcessesSectionProps {
  loaded: boolean;
  procs: IDiscoveredProcess[];
  missing: ProcessWatchItem[];
  selected: Set<string>;
  showAll: boolean;
  onShowAll: (v: boolean) => void;
  toggle: (item: WatchItem, on: boolean) => void;
}

/** Программы VPN: найденные, выбранные незапущенные и ввод имени вручную. */
export function ProcessesSection(props: IProcessesSectionProps) {
  const { loaded, procs, missing, selected, showAll, onShowAll, toggle } = props;
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
      {loaded && procs.length === 0 && missing.length === 0 && (
        <Typography.Text type="secondary">{T.empty}</Typography.Text>
      )}
      {procs.map(p => {
        const item: WatchItem = { kind: 'process', name: p.name };
        return (
          <PickerRow
            key={p.name.toLowerCase()}
            checked={selected.has(watchKey(item))}
            onChange={on => toggle(item, on)}
            title={
              <>
                {p.name} {p.knownVpn && <Tag color="blue">{T.vpn}</Tag>}
              </>
            }
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
        />
      ))}
      <ManualProcess onAdd={name => toggle({ kind: 'process', name }, true)} />
    </PickerSection>
  );
}
