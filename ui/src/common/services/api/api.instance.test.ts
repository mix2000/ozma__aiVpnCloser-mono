import { afterEach, describe, expect, it } from 'vitest';

import { errorText } from './api-errors';
import { call, listen, setFallbackBackend } from './api.instance';

describe('транспорт', () => {
  afterEach(() => setFallbackBackend(null));

  it('текст ошибки из строки, Error и объекта', () => {
    expect(errorText('Плохо')).toBe('Плохо');
    expect(errorText(new Error('x'))).toBe('x');
    expect(errorText({ a: 1 })).toBe('{"a":1}');
  });

  it('вне Tauri — подставной бэкенд, ошибка строкой → Error', async () => {
    const seen: unknown[] = [];
    setFallbackBackend({
      call: async cmd => {
        if (cmd === 'bad') throw 'Неверный адрес';
        return { cmd };
      },
      listen: (event, cb) => {
        (cb as (payload: unknown) => void)(event);
        return () => seen.push('off');
      },
    });
    await expect(call('get_state')).resolves.toEqual({ cmd: 'get_state' });
    await expect(call('bad')).rejects.toThrow('Неверный адрес');
    const off = listen<string>('state-changed', p => seen.push(p));
    off();
    expect(seen).toEqual(['state-changed', 'off']);
  });

  it('без бэкенда — понятная ошибка', async () => {
    await expect(call('get_state')).rejects.toThrow('Нет связи с программой');
  });
});
