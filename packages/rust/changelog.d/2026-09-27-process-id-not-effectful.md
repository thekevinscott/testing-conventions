**Fixed** **`std::process::id` is no longer an effectful `std` reach** (#754). The Rust isolation
rules treat all of `std::process` as effectful, which is right for `Command`, `exit` and `abort` —
they spawn, control or terminate. `id` does none of that: it reports the runner's own PID, so
there is no ambient state being read and nothing to inject a double for.

Its one idiomatic use is naming a temp directory so concurrent tests do not collide —
`env::temp_dir().join(format!("prefix-{}", process::id()))` — and `env::temp_dir` is already
carved out for exactly that reason. Flagging one half of a two-call idiom and allowing the other
made the rule arbitrary at the point consumers meet it most.

Surfaced by #739: once macro token bodies were read, this single call accounted for 16 of this
crate's 50 findings.
