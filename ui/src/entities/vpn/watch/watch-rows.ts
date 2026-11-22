import type { ICheck, WatchItem } from 'common/types';
import { CHECK_PAUSED_TEXT, CHECK_PENDING_TEXT } from './watch.consts';
import { watchKey, watchTitle } from './watch-items';

/**
 * Строки «За чем слежу»: проверки по выбранным пунктам; пока монитор их не
 * прислал — выбранное без результата.
 */
export function watchRows(watch: WatchItem[], checks: ICheck[], monitoring: boolean): ICheck[] {
  const byKey = new Map(checks.map(c => [watchKey(c.item), c]));
  return watch.map(
    item =>
      byKey.get(watchKey(item)) ?? {
        item,
        ok: null,
        label: watchTitle(item),
        detail: monitoring ? CHECK_PENDING_TEXT : CHECK_PAUSED_TEXT,
      }
  );
}
