export type PluralForms = [string, string, string];

/** Русское склонение: plural(5, ['сессия', 'сессии', 'сессий']) → 'сессий'. */
export function plural(n: number, forms: PluralForms): string {
  const m10 = n % 10;
  const m100 = n % 100;
  if (m10 === 1 && m100 !== 11) return forms[0];
  if (m10 >= 2 && m10 <= 4 && (m100 < 12 || m100 > 14)) return forms[1];
  return forms[2];
}
