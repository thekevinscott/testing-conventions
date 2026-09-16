// A type declaration alongside a runtime function — this HAS behavior, so it
// stays a colocated-test subject.
export type Version = number;

export const version = (): Version => 1;
