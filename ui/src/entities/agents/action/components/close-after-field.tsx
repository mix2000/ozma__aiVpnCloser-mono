import { Col, InputNumber, Row, Slider, Typography } from 'antd';
import { useEffect, useState } from 'react';

import type { IRange } from 'common/types';
import { clampInt } from 'common/utils';

import { CLOSE_AFTER_STEP_SEC, CLOSE_AFTER_TEXTS as T } from './close-after-field.consts';
import styles from './close-after-field.module.scss';

interface ICloseAfterFieldProps {
  value: number;
  range: IRange;
  onCommit: (sec: number) => void;
}

/** «Закрыть через» 0–600 с: ползунок и поле, сохраняется по отпусканию / Enter. */
export function CloseAfterField({ value, range, onCommit }: ICloseAfterFieldProps) {
  const [v, setV] = useState(value);
  useEffect(() => setV(value), [value]);

  const commit = (x: number | null) => {
    const n = clampInt(x, range.min, range.max);
    setV(n);
    if (n !== value) onCommit(n);
  };

  return (
    <div className={styles.field}>
      <div className={styles.label}>{T.label}</div>
      <Row gutter={12} align="middle">
        <Col flex="auto">
          <Slider
            min={range.min}
            max={range.max}
            step={CLOSE_AFTER_STEP_SEC}
            value={v}
            onChange={setV}
            onChangeComplete={commit}
            tooltip={{ formatter: T.tooltip }}
          />
        </Col>
        <Col>
          <InputNumber
            min={range.min}
            max={range.max}
            value={v}
            onChange={x => setV(x ?? 0)}
            onBlur={() => commit(v)}
            onPressEnter={() => commit(v)}
            suffix={T.unit}
            className={styles.input}
            aria-label={T.inputLabel}
          />
        </Col>
      </Row>
      <Typography.Text type="secondary" className={styles.hint}>
        {v === 0 ? T.now : T.later(v)}
      </Typography.Text>
    </div>
  );
}
