/** Return the value unchanged. */
export function described(value: number): number {
  // The identity is the whole contract.
  //
  // A blank line follows this comment block.

  return value;
}

export function compute(value: number): number {
  const scaled = value * 2;
  return scaled + 1;
}
