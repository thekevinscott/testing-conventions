/** The stub `package.json` that claims `name` on the registry, replaced by the first real
 * release. */
export function bootstrapManifest(name: string): Record<string, unknown> {
  return {
    name,
    version: '0.0.0-bootstrap',
    description: 'Bootstrap stub; replaced on first real publish via OIDC trusted publisher.',
    license: 'MIT',
    repository: {
      type: 'git',
      url: 'git+https://github.com/thekevinscott/testing-conventions.git',
    },
  };
}
