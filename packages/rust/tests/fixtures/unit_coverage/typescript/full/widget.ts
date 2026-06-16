export function classify(n: number): string {
  if (n > 0) return 'positive';
  if (n < 0) return 'negative';
  return 'zero';
}
