import { Button, Input, Space } from 'antd';
import { useState } from 'react';

import { PROCESSES_SECTION_TEXTS as T } from './processes-section.consts';
import styles from './processes-section.module.scss';

/** Ввод имени программы VPN вручную. */
export function ManualProcess({ onAdd }: { onAdd: (name: string) => void }) {
  const [manual, setManual] = useState('');
  const add = () => {
    const name = manual.trim();
    if (!name) return;
    onAdd(name);
    setManual('');
  };
  return (
    <Space.Compact className={styles.manual}>
      <Input
        value={manual}
        onChange={e => setManual(e.target.value)}
        onPressEnter={add}
        placeholder={T.manualPlaceholder}
        aria-label={T.manualLabel}
      />
      <Button onClick={add} disabled={!manual.trim()}>
        {T.add}
      </Button>
    </Space.Compact>
  );
}
