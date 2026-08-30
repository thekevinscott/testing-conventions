export function first(value: number): number {
  const total = value + 1;
  return total;
}

export function second(value: number): number {
  const total = value + 2;
  return total;
}

const third = function (value: number): number {
  const total = value + 3;
  return total;
};

export { third };
