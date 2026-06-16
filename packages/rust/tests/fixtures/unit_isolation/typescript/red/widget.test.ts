import { describe, expect, it, vi } from 'vitest';

import { makeWidget } from './widget'; // unit under test — never a collaborator
import { format } from './formatter'; // first-party collaborator, NOT mocked → violation
import { chunk } from 'lodash'; // external collaborator, NOT mocked → violation
import { log } from './logger'; // a collaborator, but mocked below → fine

vi.mock('./logger');

describe('makeWidget', () => {
  it('builds a widget', () => {
    const widget = makeWidget({ format, chunk, log });
    expect(widget).toBeTruthy();
  });
});
