import { Space, Switch } from 'antd';
import type { ReactNode } from 'react';

import styles from './picker-section.module.scss';

interface IShowAll {
  label: string;
  ariaLabel: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}

interface IPickerSectionProps {
  title: string;
  /** Переключатель «показать все» справа от заголовка. */
  showAll?: IShowAll;
  children: ReactNode;
}

/** Раздел окна выбора: заголовок и строки. */
export function PickerSection({ title, showAll, children }: IPickerSectionProps) {
  return (
    <div className={styles.section}>
      <div className={styles.sectionTitle}>
        {title}
        {showAll && (
          <Space size={6} className={styles.showAll}>
            {showAll.label}
            <Switch
              size="small"
              checked={showAll.checked}
              onChange={showAll.onChange}
              aria-label={showAll.ariaLabel}
            />
          </Space>
        )}
      </div>
      {children}
    </div>
  );
}
