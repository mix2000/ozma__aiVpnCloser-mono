import type { ReactNode } from 'react';

import styles from './setting-row.module.scss';

interface ISettingRowProps {
  title: string;
  hint?: string;
  children: ReactNode;
  last?: boolean;
}

/** Строка настройки: название и пояснение слева, переключатель справа. */
export function SettingRow({ title, hint, children, last }: ISettingRowProps) {
  return (
    <div className={`${styles.row} ${last ? styles.last : ''}`}>
      <div className={styles.text}>
        <div>{title}</div>
        {hint && <div className={styles.hint}>{hint}</div>}
      </div>
      <div className={styles.control}>{children}</div>
    </div>
  );
}
