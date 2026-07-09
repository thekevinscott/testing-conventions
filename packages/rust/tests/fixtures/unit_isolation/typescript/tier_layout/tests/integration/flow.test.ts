import { describe, expect, it } from 'vitest';

// An integration test: first-party code runs for real, deliberately unmocked.
// The unit-suite isolation rule holds no claim under `<package root>/tests/`.
import { makeWidget } from '../../src/widget';

describe('flow (integration)', () => {
  it('builds a widget', () => {
    expect(makeWidget()).toBeTruthy();
  });
});
