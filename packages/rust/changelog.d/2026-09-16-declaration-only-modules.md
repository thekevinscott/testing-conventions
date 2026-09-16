**Changed** `unit colocated-test` skips a declaration-only module — a Python or TypeScript file
with no function and no control flow anywhere in it — for presence and co-change alike. A
re-export barrel `__init__.py` / `index.ts`, a constants module, an `Enum`, an enum-only or
`export const` module, and an `as const` object need no colocated test and no exemption; the
parser decides from the file's contents. A file holding a `def`, a lambda, an arrow, a method, an
`if`, a ternary, or any other branch or loop is a subject as before. Rust is unchanged. See
`../migrations.d/2026-09-16-declaration-only-modules.md`.
