import { expect, test } from '@playwright/test';

import { chooseRecommendedVpn } from './helpers';

test('выбор «что считать VPN»: туннели с программой-владельцем и проверка маршрута', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByText('Что считать включённым VPN?')).toBeVisible();
  await page.getByRole('button', { name: 'Выбрать…' }).first().click();
  const modal = page.locator('.ant-modal');
  await expect(modal.getByText('(sing-box)')).toBeVisible();
  await expect(modal.getByText('(вероятно OpenVPN)')).toBeVisible();
  await modal.getByRole('button', { name: 'Отмена' }).click();

  await chooseRecommendedVpn(page);
  await expect(page.getByText('Интерфейс utun233 (172.19.0.1/30)')).toBeVisible();
  await expect(page.getByText('(sing-box)')).toBeVisible();
  await expect(page.getByText('Трафик к агентам идёт через VPN')).toBeVisible();
  await expect(page.getByText('Что считать включённым VPN?')).toBeHidden();
});
