// A type declaration alongside a runtime `const` — this HAS behavior, so it
// stays a colocated-test subject even after type-only modules are recognized.
export type Version = number;

export const version: Version = 1;
