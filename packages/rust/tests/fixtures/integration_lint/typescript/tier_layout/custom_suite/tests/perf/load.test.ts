import { describe, expect, it, vi } from 'vitest';

import { greet } from '../../src/widget';

vi.mock('../../src/widget', () => ({
  greet: vi.fn(),
}));

describe('load (perf)', () => {
  it('greets', () => {
    expect(greet('Ada')).toBeDefined();
  });
});
