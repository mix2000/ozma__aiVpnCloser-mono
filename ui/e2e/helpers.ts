import { expect, type Page } from '@playwright/test';

/** Открыть окно и выбрать рекомендуемое «что считать VPN». */
export async function chooseRecommendedVpn(page: Page) {
  await page.goto('/');
  await page.getByRole('button', { name: 'Выбрать…' }).first().click();
  const modal = page.locator('.ant-modal');
  await expect(modal.getByText('Что считать включённым VPN')).toBeVisible();
  await modal.getByRole('button', { name: 'Выбрать рекомендуемое' }).click();
  await modal.getByRole('button', { name: 'Сохранить' }).click();
  await expect(modal).toBeHidden();
}

/** Переключить «VPN есть» в моке (как из консоли: __mock.setVpn). */
export async function setVpn(page: Page, up: boolean) {
  await page.evaluate(
    v => (globalThis as unknown as { __mock: { setVpn(x: boolean): void } }).__mock.setVpn(v),
    up
  );
}

/** Заголовок статуса в шапке (тот же текст бывает и в журнале — ищем только в шапке). */
export function statusTitle(page: Page) {
  return page.locator('[data-status]').first();
}
