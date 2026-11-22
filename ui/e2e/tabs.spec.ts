import { expect, test } from '@playwright/test';

test('вкладки «Действие» и «Настройки»', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('tab', { name: 'Действие' }).click();
  await expect(page.getByText('— 3 сессии (+3 вспомогательных процесса)')).toBeVisible();
  await page.getByRole('button', { name: /Кого затронет сейчас/ }).click();
  await expect(page.getByText('Claude Code · claude · pid 4101')).toBeVisible();

  const dryRun = page.locator('.ant-tabs-tabpane-active').getByRole('switch').last();
  await expect(dryRun).toBeChecked();
  await dryRun.click();
  await expect(page.locator('.ant-tag', { hasText: 'Только смотреть' })).toBeHidden();

  await page.getByRole('tab', { name: 'Настройки' }).click();
  await expect(page.getByText('Запускать при входе в систему')).toBeVisible();
  await expect(page.getByLabel('Интервал проверки, мс')).toHaveValue('1000');
});
