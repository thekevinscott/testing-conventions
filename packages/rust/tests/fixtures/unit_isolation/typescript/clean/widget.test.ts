import { describe, expect, it, vi } from 'vitest';
import type { WidgetOptions } from './types'; // type-only — erased, not a runtime dep

import { makeWidget } from './widget'; // unit under test
import { format } from './formatter';
import { chunk } from 'lodash';
import { log } from './logger';

// Every collaborator is mocked; the unit under test runs for real.
vi.mock('./formatter');
vi.mock('lodash');
vi.mock('./logger');

describe('makeWidget', () => {
  it('builds a widget', () => {
    const opts: WidgetOptions = { format, chunk, log };
    expect(makeWidget(opts)).toBeTruthy();
  });
});
