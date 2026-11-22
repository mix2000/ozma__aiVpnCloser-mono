import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { installFallback, installMock, renderWithProviders } from 'common/test';

import { MainPage } from './main-page';

describe('главная страница', () => {
  it('загрузка → статус и вкладки; переключение вкладок', async () => {
    installMock();
    renderWithProviders(<MainPage />);
    expect(await screen.findByText('Наблюдение выключено')).toBeInTheDocument();
    expect(screen.getByText('За чем слежу')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: 'Настройки' }));
    expect(await screen.findByText('Запускать при входе в систему')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: 'Журнал' }));
    expect(await screen.findByText('Программа запущена (мок).')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'выберите, что считать VPN' }));
    expect(await screen.findByText('За чем слежу')).toBeVisible();
  });

  it('нет связи с программой', async () => {
    installFallback({ call: () => Promise.reject('нет ответа'), listen: () => () => undefined });
    renderWithProviders(<MainPage />);
    expect(await screen.findByText('Не удалось связаться с программой')).toBeInTheDocument();
    expect(screen.getAllByText('нет ответа').length).toBeGreaterThan(0);
  });
});
