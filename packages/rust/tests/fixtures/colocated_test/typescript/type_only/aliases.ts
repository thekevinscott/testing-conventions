// Type-only module built on `import type` + `export type` — still zero runtime.
import type { Shape } from './shape';

export type Wrapped = Shape;
export type { Id } from './shape';
