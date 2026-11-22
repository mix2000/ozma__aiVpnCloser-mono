import { CheckCircleFilled, CloseCircleFilled, QuestionCircleFilled } from '@ant-design/icons';

import { CHECK_ICON_LABELS } from './check-icon.consts';
import styles from './check-icon.module.scss';

/** ✓ / ✗ / ? по результату проверки. */
export function CheckIcon({ ok }: { ok: boolean | null }) {
  if (ok === true)
    return (
      <CheckCircleFilled
        className={`${styles.icon} ${styles.ok}`}
        aria-label={CHECK_ICON_LABELS.ok}
      />
    );
  if (ok === false)
    return (
      <CloseCircleFilled
        className={`${styles.icon} ${styles.fail}`}
        aria-label={CHECK_ICON_LABELS.fail}
      />
    );
  return (
    <QuestionCircleFilled
      className={`${styles.icon} ${styles.unknown}`}
      aria-label={CHECK_ICON_LABELS.unknown}
    />
  );
}
