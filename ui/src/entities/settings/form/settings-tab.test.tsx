import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from 'common/test';

import { MOCK_LIMITS } from 'mocks/mock-data';

import { DEFAULT_CONFIG } from '../config/config.consts';
import { SettingsTab } from './settings-tab';

describe('вкладка «Настройки»', () => {
  it('переключатели и числа сохраняются', async () => {
    const save = vi.fn().mockResolvedValue(true);
    renderWithProviders(<SettingsTab config={DEFAULT_CONFIG} limits={MOCK_LIMITS} save={save} />);
    const [autostart, notify] = screen.getAllByRole('switch');
    await userEvent.click(autostart);
    expect(save).toHaveBeenCalledWith({ autostart: true });
    await userEvent.click(notify);
    expect(save).toHaveBeenCalledWith({ notify: false });
    const confirm = screen.getByLabelText('Подтверждений');
    fireEvent.change(confirm, { target: { value: '3' } });
    fireEvent.blur(confirm);
    expect(save).toHaveBeenCalledWith({ confirmFailures: 3 });
    expect(screen.getByText(/Закрыть окно — программа останется в/)).toBeInTheDocument();
  });

  it('интервал и ожидание; подпись про трей на Windows', () => {
    const save = vi.fn().mockResolvedValue(true);
    const platform = vi.spyOn(navigator, 'platform', 'get').mockReturnValue('Win32');
    renderWithProviders(<SettingsTab config={DEFAULT_CONFIG} limits={MOCK_LIMITS} save={save} />);
    const poll = screen.getByLabelText('Интервал проверки, мс');
    fireEvent.change(poll, { target: { value: '30000' } });
    fireEvent.blur(poll);
    expect(save).toHaveBeenCalledWith({ pollMs: 30000 });
    const grace = screen.getByLabelText('Ожидание перед принудительным закрытием');
    fireEvent.change(grace, { target: { value: '10' } });
    fireEvent.blur(grace);
    expect(save).toHaveBeenCalledWith({ killGraceSec: 10 });
    expect(screen.getByText(/останется в трее/)).toBeInTheDocument();
    platform.mockRestore();
  });
});
