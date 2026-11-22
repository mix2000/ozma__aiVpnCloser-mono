import { InputNumber } from 'antd';
import { useEffect, useState } from 'react';
import styles from './number-setting.module.scss';

interface INumberSettingProps {
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  label: string;
  onCommit: (v: number) => void;
}

/** Число, которое сохраняется по выходу из поля / Enter (в пределах min…max). */
export function NumberSetting({
  value,
  min,
  max,
  step,
  unit,
  label,
  onCommit,
}: INumberSettingProps) {
  const [v, setV] = useState<number>(value);
  useEffect(() => setV(value), [value]);
  const commit = () => {
    const n = Math.max(min, Math.min(max, Math.round(v)));
    setV(n);
    if (n !== value) onCommit(n);
  };
  return (
    <InputNumber
      min={min}
      max={max}
      step={step}
      value={v}
      onChange={x => setV(x ?? value)}
      onBlur={commit}
      onPressEnter={commit}
      suffix={unit}
      className={styles.input}
      aria-label={label}
    />
  );
}
