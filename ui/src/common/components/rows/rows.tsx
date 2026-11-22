import type { ReactNode } from 'react';

import styles from './rows.module.scss';

interface IRowsProps<T> {
  items: T[];
  rowKey: (item: T) => string | number;
  render: (item: T) => ReactNode;
  empty?: ReactNode;
  bordered?: boolean;
}

/** Простой список строк (List в antd 6 устарел). */
export function Rows<T>({ items, rowKey, render, empty, bordered }: IRowsProps<T>) {
  if (items.length === 0) return <div className={styles.empty}>{empty}</div>;
  return (
    <div className={`${styles.list} ${bordered ? styles.bordered : ''}`}>
      {items.map(it => (
        <div className={styles.row} key={rowKey(it)}>
          {render(it)}
        </div>
      ))}
    </div>
  );
}
