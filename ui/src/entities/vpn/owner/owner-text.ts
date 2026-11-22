import type { IIfOwner } from 'common/types';
import { OWNER_TEXTS as T } from './owner.consts';

/** Подпись владельца в скобках: «(sing-box)», «(вероятно OpenVPN)». */
export function ownerText(owner: IIfOwner): string {
  return `(${owner.confidence === 'likely' ? T.likelyPrefix : ''}${owner.name})`;
}
