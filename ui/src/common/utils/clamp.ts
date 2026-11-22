/** Целое в пределах min…max (пусто — min). */
export function clampInt(v: number | null | undefined, min: number, max: number): number {
  return Math.max(min, Math.min(max, Math.round(v ?? min)));
}
