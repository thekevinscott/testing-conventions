export function build(values: number[]): number[] {
  const inner = (value: number): number => {
    const doubled = value * 2;
    return doubled;
  };
  return values.map(inner).sort((left, right) => right - left);
}
