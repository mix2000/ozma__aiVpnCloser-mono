import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { IAppState } from 'common/types';
import { StatusHeader } from './status-header';

const base: IAppState = {
  monitoring: true,
  dryRun: true,
  status: 'countdown',
  countdownLeftSec: 23,
  checks: [],
  agents: [],
  lastChangeAt: null,
  color: 'yellow',
  title: 'VPN пропал — закрою через 23 с',
};

describe('шапка статуса', () => {
  it('отсчёт: цвет, текст, бейдж «Только смотреть», переключатель', async () => {
    const onToggle = vi.fn();
    const { container } = render(
      <StatusHeader
        state={base}
        canMonitor
        switching={false}
        onToggle={onToggle}
        onChooseWatch={vi.fn()}
      />
    );
    expect(screen.getByText('VPN пропал — закрою через 23 с')).toBeInTheDocument();
    expect(container.querySelector('[data-status="yellow"]')).not.toBeNull();
    expect(screen.getByText('Только смотреть')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('switch', { name: 'Следить' }));
    expect(onToggle).toHaveBeenCalledWith(false, expect.anything());
  });

  it('не выбрано, за чем следить: ссылка и выключенный переключатель', async () => {
    const onChoose = vi.fn();
    render(
      <StatusHeader
        state={{
          ...base,
          monitoring: false,
          dryRun: false,
          status: 'paused',
          color: 'gray',
          title: 'Наблюдение выключено',
        }}
        canMonitor={false}
        switching={false}
        onToggle={vi.fn()}
        onChooseWatch={onChoose}
      />
    );
    expect(screen.getByText('Наблюдение выключено')).toBeInTheDocument();
    expect(screen.getByRole('switch', { name: 'Следить' })).toBeDisabled();
    expect(screen.queryByText('Только смотреть')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'выберите, что считать VPN' }));
    expect(onChoose).toHaveBeenCalled();
  });
});
