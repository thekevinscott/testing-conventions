export function alpha(value: number): number {
  const scaled = value * 2;
  const total = scaled + 1;
  return total;
}

export function beta(value: number): number {
  const scaled = value * 3;
  const total = scaled + 2;
  return total;
}
