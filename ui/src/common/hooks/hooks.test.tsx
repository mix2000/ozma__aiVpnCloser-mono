import { renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { APP_NAME } from 'common/consts';

import { useDocumentTitle } from './use-document-title';

describe('общие хуки', () => {
  it('заголовок окна', () => {
    renderHook(() => useDocumentTitle(APP_NAME));
    expect(document.title).toBe('ozma AI VPN Closer');
  });
});
