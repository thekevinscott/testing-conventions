**Changed** `unit mutation --language rust` drops cargo-mutants' whole-body (`FnValue`) mutant on a
**subprocess seam** — a branch-free function that runs a `std::process::Command`, directly or
through another function in the same file (#752). Calling one spawns a process, so no unit test can
observe its body being replaced by `Ok(String::new())`, and the tiers that can are outside the
unit-tier `--lib --bins` scoping. A body that decides *and* spawns is not a seam: it keeps every
mutant, including the whole-body one, because the decision becomes assertable once it moves into a
function that returns the argv. Only the `FnValue` genre is dropped; a mutation inside a seam is
still judged. Rust only — the drop keys on a cargo-mutants genre, not on a property of the language.
See [`../migrations.d/2026-09-27-rust-subprocess-seam.md`](../migrations.d/2026-09-27-rust-subprocess-seam.md).
