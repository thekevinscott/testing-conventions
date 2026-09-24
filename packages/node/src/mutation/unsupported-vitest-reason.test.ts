import { describe, expect, it } from 'vitest';

import { unsupportedVitestReason } from './unsupported-vitest-reason.js';

describe('unsupportedVitestReason', () => {
  it('refuses vitest 5, naming the version and the pin that works', () => {
    const reason = unsupportedVitestReason('5.0.1');

    expect(reason).toContain('vitest 5.0.1');
    expect(reason).toContain('^4');
  });

  it('refuses a major above the first unsupported one', () => {
    expect(unsupportedVitestReason('6.2.0')).not.toBeNull();
  });

  it('refuses a prerelease of an unsupported major', () => {
    expect(unsupportedVitestReason('5.0.0-beta.3')).not.toBeNull();
  });

  it('admits vitest 4, the newest major the runner still judges mutants on', () => {
    expect(unsupportedVitestReason('4.1.11')).toBeNull();
  });

  it('admits the older supported majors', () => {
    expect(unsupportedVitestReason('3.2.4')).toBeNull();
    expect(unsupportedVitestReason('2.0.0')).toBeNull();
  });

  it('admits an absent vitest, leaving that failure to the runner', () => {
    expect(unsupportedVitestReason(null)).toBeNull();
  });

  it('admits an unparseable version rather than guessing', () => {
    expect(unsupportedVitestReason('not-a-version')).toBeNull();
  });
});
