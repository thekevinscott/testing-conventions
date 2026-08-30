const CONSTANT = 3;

export function normalize(value: number): number {
  const scaled = value * CONSTANT;
  return scaled;
}

export const double = (value: number): number => value * 2;

export function triple(value: number): number {
  return value * 3;
}
