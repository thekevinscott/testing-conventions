/** The npm package names a comma-separated `packages` input carries, trimmed and in order. An
 * input naming none throws: a bootstrap run that publishes nothing must say so, not pass green. */
export function bootstrapNames(packages: string): string[] {
  const names = packages
    .split(',')
    .map((name) => name.trim())
    .filter((name) => name.length > 0);
  if (names.length === 0) {
    throw new Error(`\`packages\` names nothing to bootstrap: ${JSON.stringify(packages)}`);
  }
  return names;
}
