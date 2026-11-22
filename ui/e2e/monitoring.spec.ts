import { expect, test } from '@playwright/test';

import { chooseRecommendedVpn, setVpn, statusTitle } from './helpers';

test('«Следить»: VPN пропал → отсчёт, вернулся → включён, всё в журнале', async ({ page }) => {
  await chooseRecommendedVpn(page);
  await expect(statusTitle(page)).toContainText('Наблюдение выключено');
  await page.getByRole('switch', { name: 'Следить' }).click();
  await expect(statusTitle(page)).toContainText('VPN включён');

  await setVpn(page, false);
  await expect(statusTitle(page)).toContainText(/VPN пропал — закрою через \d+ с/);
  await setVpn(page, true);
  await expect(statusTitle(page)).toContainText('VPN включён');

  await page.getByRole('tab', { name: 'Журнал' }).click();
  await expect(page.getByText(/VPN пропал — заморозил бы агентов/)).toBeVisible();
  await expect(page.getByText('VPN вернулся — агенты продолжают.')).toBeVisible();

  await page.getByRole('switch', { name: 'Следить' }).click();
  await expect(statusTitle(page)).toContainText('Наблюдение выключено');
});
