import { describe, expect, it } from 'vitest';

import { bootstrapManifest } from './bootstrap-manifest.js';

describe('bootstrapManifest', () => {
  it('names the package it was asked for', () => {
    expect(bootstrapManifest('@testing-conventions/darwin-arm64')).toMatchObject({
      name: '@testing-conventions/darwin-arm64',
    });
  });

  it('publishes a prerelease version, so the first real release takes `latest`', () => {
    expect(bootstrapManifest('testing-conventions').version).toBe('0.0.0-bootstrap');
  });

  it('carries the license and repository npm shows on the stub', () => {
    expect(bootstrapManifest('testing-conventions')).toMatchObject({
      description: 'Bootstrap stub; replaced on first real publish via OIDC trusted publisher.',
      license: 'MIT',
      repository: {
        type: 'git',
        url: 'git+https://github.com/thekevinscott/testing-conventions.git',
      },
    });
  });
});
