import pkg from '../package.json';

export const VERSION: string = pkg.version;

export function isPositive(n: number): boolean {
  return n > 0;
}
