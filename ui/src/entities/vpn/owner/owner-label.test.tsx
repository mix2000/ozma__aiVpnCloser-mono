import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { OwnerLabel } from './owner-label';

describe('OwnerLabel', () => {
  it('точно — обычным текстом, вероятно — серым, нет — ничего', () => {
    const { rerender, container } = render(
      <OwnerLabel owner={{ name: 'sing-box', pid: 812, confidence: 'exact' }} />
    );
    expect(screen.getByText('(sing-box)')).toHaveAttribute('title', 'pid 812');
    rerender(<OwnerLabel owner={{ name: 'OpenVPN', confidence: 'likely' }} />);
    const likely = screen.getByText('(вероятно OpenVPN)');
    expect(likely.className).toMatch(/secondary/);
    rerender(<OwnerLabel owner={null} />);
    expect(container).toBeEmptyDOMElement();
  });
});
