import { describe, expect, it } from 'vitest';

import { greet } from './widget';

describe('greet', () => {
  it('greets', () => {
    expect(greet('Ada')).toBe('Hello, Ada!');
  });
});
