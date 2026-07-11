// Type-only module: only type/interface declarations, no runtime code.
// Compiles to zero JS, so it has no behavior to unit-test.
export interface Shape {
  kind: string;
  sides: number;
}

export type Id = string;
