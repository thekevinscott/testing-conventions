export function encode(value: number): number {
  const total = value + 1;
  return total;
}

export function decode(value: number): number {
  const total = value - 1;
  return total;
}
