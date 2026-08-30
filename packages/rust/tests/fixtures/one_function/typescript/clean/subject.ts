export function ratio(numerator: number, denominator: number): number {
  const total = numerator + denominator;
  return Math.floor(total / 2);
}
