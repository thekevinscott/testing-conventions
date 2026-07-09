// A suite helper with real logic and no colocated twin: the suite tiers belong
// to the integration checks, so the colocated-unit rule holds no claim here.
export function fixturePath(name: string): string {
  return `fixtures/${name}.json`;
}
