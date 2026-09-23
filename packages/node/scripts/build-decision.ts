/** `true` when `target` names a row the JS shim builds on: empty, `main`, or `noarch`. A
 * per-triple row is the engine staging the binary, so the caller exits without building. */
export function shouldBuildShim(target: string): boolean {
  return target === '' || target === 'main' || target === 'noarch';
}
