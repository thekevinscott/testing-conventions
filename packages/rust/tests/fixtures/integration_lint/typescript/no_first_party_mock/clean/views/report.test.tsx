import { describe, expect, it, vi } from 'vitest';

import { Report } from '../src/report';

// A Node built-in and a scoped third-party package — both allowed in an
// integration test. The JSX below exercises `.tsx` parsing.
vi.mock('node:path');
vi.mock('@aws-sdk/client-s3');

describe('<Report /> (integration)', () => {
  it('renders a heading', () => {
    const element = <Report title="Q2" />;
    expect(element).toBeTruthy();
  });
});
