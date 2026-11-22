import { Modal } from 'antd';
import { useState } from 'react';

import type { IConfig, WatchItem } from 'common/types';

import { suggestWatch } from '../watch/watch-items';
import { FoundSections, PickerToolbar, RouteSection } from './components';
import { useDiscovery } from './hooks';
import { usePickerState } from './use-picker-state';
import { WATCH_PICKER_TEXTS as T } from './watch-picker.consts';
import styles from './watch-picker.module.scss';

interface IWatchPickerProps {
  config: IConfig;
  onCancel: () => void;
  onSave: (watch: WatchItem[], hosts: string[]) => Promise<void>;
}

/** Окно выбора «что считать VPN»: найденное на машине, чекбоксы, хосты. */
export function WatchPicker({ config, onCancel, onSave }: IWatchPickerProps) {
  const { found, loadError, loading, refresh } = useDiscovery();
  const p = usePickerState(config);
  const [saving, setSaving] = useState(false);

  const submit = async () => {
    setSaving(true);
    try {
      await onSave(p.watch, p.hostsToSave);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal
      open
      width={640}
      title={T.title}
      okText={T.save}
      cancelText={T.cancel}
      onCancel={onCancel}
      onOk={() => void submit()}
      confirmLoading={saving}
      okButtonProps={{ disabled: p.watch.length === 0 }}
      destroyOnHidden
    >
      <div className={styles.body}>
        <PickerToolbar
          loading={loading}
          error={loadError}
          onRefresh={() => void refresh()}
          onSuggest={found ? () => p.setWatch(suggestWatch(found)) : null}
        />
        <RouteSection
          on={p.selected.has('route')}
          onToggle={on => p.toggle({ kind: 'route' }, on)}
          hostsText={p.hostsText}
          hostsCount={p.hostsCount}
          onHostsText={p.setHostsText}
          onResetHosts={p.resetHosts}
        />
        <FoundSections found={found} watch={p.watch} selected={p.selected} toggle={p.toggle} />
      </div>
    </Modal>
  );
}
