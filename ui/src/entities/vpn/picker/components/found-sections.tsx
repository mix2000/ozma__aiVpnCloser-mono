import { useState } from 'react';

import type { IDiscovery, WatchItem } from 'common/types';

import {
  missingInterfaces,
  missingProcesses,
  pickerInterfaces,
  pickerProcesses,
} from '../picker-lists';
import { InterfacesSection } from './interfaces-section';
import { ProcessesSection } from './processes-section';
import { SystemVpnsSection } from './system-vpns-section';

interface IFoundSectionsProps {
  found: IDiscovery | null;
  watch: WatchItem[];
  selected: Set<string>;
  toggle: (item: WatchItem, on: boolean) => void;
}

/** Найденное на машине: интерфейсы, программы VPN, системные VPN. */
export function FoundSections({ found, watch, selected, toggle }: IFoundSectionsProps) {
  const [allIfaces, setAllIfaces] = useState(false);
  const [allProcs, setAllProcs] = useState(false);
  return (
    <>
      <InterfacesSection
        loaded={found !== null}
        ifaces={pickerInterfaces(found, selected, allIfaces)}
        missing={missingInterfaces(watch, found)}
        selected={selected}
        showAll={allIfaces}
        onShowAll={setAllIfaces}
        toggle={toggle}
      />
      <ProcessesSection
        loaded={found !== null}
        procs={pickerProcesses(found, selected, allProcs)}
        missing={missingProcesses(watch, found)}
        selected={selected}
        showAll={allProcs}
        onShowAll={setAllProcs}
        toggle={toggle}
      />
      <SystemVpnsSection vpns={found?.systemVpns ?? null} selected={selected} toggle={toggle} />
    </>
  );
}
