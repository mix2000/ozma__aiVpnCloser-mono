import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { NumberSetting } from './number-setting';
import { Rows } from './rows';
import { SettingRow } from './setting-row';

describe('общие компоненты', () => {
  it('Rows: пусто и строки', () => {
    const { rerender } = render(
      <Rows items={[]} rowKey={(x: number) => x} render={x => x} empty="никого" />
    );
    expect(screen.getByText('никого')).toBeInTheDocument();
    rerender(
      <Rows items={[1, 2]} rowKey={x => x} render={x => <span>строка {x}</span>} bordered />
    );
    expect(screen.getAllByText(/строка/)).toHaveLength(2);
  });

  it('SettingRow: название, пояснение, управление', () => {
    render(
      <SettingRow title="Уведомления" hint="Сообщать" last>
        <button>вкл</button>
      </SettingRow>
    );
    expect(screen.getByText('Уведомления')).toBeInTheDocument();
    expect(screen.getByText('Сообщать')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'вкл' })).toBeInTheDocument();
  });

  it('NumberSetting: сохраняет по выходу из поля, без изменений — нет', () => {
    const onCommit = vi.fn();
    render(
      <NumberSetting
        value={1000}
        min={200}
        max={10000}
        step={100}
        unit="мс"
        label="Интервал"
        onCommit={onCommit}
      />
    );
    const input = screen.getByLabelText('Интервал');
    fireEvent.change(input, { target: { value: '2500' } });
    fireEvent.blur(input);
    expect(onCommit).toHaveBeenCalledWith(2500);
    onCommit.mockClear();
    fireEvent.change(input, { target: { value: '1000' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onCommit).not.toHaveBeenCalled();
  });
});
