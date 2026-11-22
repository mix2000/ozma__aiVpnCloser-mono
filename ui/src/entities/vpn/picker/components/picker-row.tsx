import { Checkbox } from 'antd';
import type { ReactNode } from 'react';

import styles from './picker-row.module.scss';

interface IPickerRowProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  title: ReactNode;
  /** Вторая строка мелким шрифтом; без неё заголовок идёт прямо в строку. */
  sub?: ReactNode;
}

/** Строка выбора: чекбокс и описание пункта. */
export function PickerRow({ checked, onChange, title, sub }: IPickerRowProps) {
  return (
    <div className={styles.row}>
      <Checkbox checked={checked} onChange={e => onChange(e.target.checked)} />
      <div className={styles.rowText}>
        {sub === undefined ? (
          title
        ) : (
          <>
            <div>{title}</div>
            <div className={styles.rowSub}>{sub}</div>
          </>
        )}
      </div>
    </div>
  );
}
