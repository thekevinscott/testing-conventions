# Rust — code smells / red flags

**Review-only.** This catalog is for reading agent output, not for generating code. Naming bad patterns inside a generation context primes them — load this doc when you're reviewing, not when you're writing.

## Code smells / red flags in agent output

### Critical (almost always wrong)
| Smell | Why it's bad | What to ask |
|---|---|---|
| `.unwrap()` in non-test code | Panics on error, no recovery | "What should happen if this fails?" |
| `.expect("...")` outside tests | Same; very slightly better with message | Same |
| `unsafe { ... }` in business logic | Bypasses safety. Should be confined to FFI bindings, std-library internals | "What's the underlying constraint? Could this be done safely?" |
| Tests pass but `cargo clippy` fails | Agent skipped the lint check | Run clippy; address every warning |

### Style smells (often wrong)
| Smell | Why it's bad |
|---|---|
| `.clone()` on every line | Agent dodging the borrow checker. Each clone allocates. |
| Reflexive `Arc<Mutex<T>>` | Over-defensive. Often a borrow suffices. |
| `Box<T>` everywhere | Usually unnecessary; only needed for trait objects, recursive types, large stack values. |
| Custom `macro_rules!` | Almost always overcomplicated. Use a function, trait, or existing crate. |
| `make()`, `create()`, `build_new()` constructors | Non-idiomatic. Use `new`, `with_*`, `from`, `try_new`. |
| `String` parameter where `&str` would work | Forces caller to allocate. |
| `Vec<T>` parameter where `&[T]` would work | Same. |
| `Box<dyn Error>` returned from a library | Should be a typed error via `thiserror`. |
| `let mut x` that's never reassigned | Drop the `mut`. (clippy will catch.) |
| Missing `#[derive(Debug)]` on public types | Convention is to derive Debug everywhere. |
| Magic numbers / strings | Should be named `const FOO: usize = 42;` |
| Tests only in `tests/` when `#[cfg(test)] mod tests` would work | Agent treating Rust like Python. Inline `mod tests` is the Rust way. |
| `?` everywhere with no `.with_context()` | Errors propagate but lose context. |

### Subtle smells
- **No clippy run.** Agent's diff should pass `cargo clippy` clean (or with intentional `#[allow(...)]`).
- **Trait imports in `use` for no obvious reason.** Often intentional — trait must be in scope for methods. Not a smell.
- **`type` aliases used like a renamed module.** Sometimes valid; sometimes obscuring.
- **`#[allow(dead_code)]` without explanation.** What's the dead code, and why?
- **A path derived from an `ancestors()` walk handed to `Command::current_dir`.** `Path::ancestors` ends at `""` for a relative input, and `""` silently behaves like the cwd for `join`/`is_file` — so a walk looking for a manifest can legitimately return an empty path. `current_dir("")` then fails with ENOENT, which reads as "the binary is missing" rather than "the directory is". Normalise empty to `.` at the spawn site, and check the directory exists before spawning so the two failures stay distinguishable. Ask: does any test cover a **relative** input path? Absolute temp dirs in every test are how this class of bug ships.
- **A `.context()` on a spawn that names one cause the OS error can't distinguish.** ``.context("running the adapter (is `node` installed?)")`` reads as a diagnosis, but `Command::output()` returns the same ENOENT whether the program is off PATH *or* the working directory doesn't exist — and never for a bad argument path. A context string that commits to one of those is wrong half the time, and it sends the reader to check the wrong thing. State what the spawn *did*, naming every path it used — interpreter, entry point, working directory — and leave the cause to the OS error the chain already carries. Ask: which distinct failures reach this one message, and does it name a path the reader can go look at?

### When `unsafe` IS legitimate
- FFI bindings to C libraries (PyO3, napi-rs, native libraries)
- Implementing primitive data structures (rare in application code)
- Hardware access (embedded, OS kernel)
- Specific verified-by-hand performance optimizations

If `unsafe` appears outside these categories, treat it as a strong red flag.
