**Added** `packaging <root>` discovers the built distributions under a directory. The root is
searched recursively for `*.whl` / `*.tar.gz` / `*.tgz` / `*.crate` and each is checked under the
language its extension names — Python, Python, TypeScript, Rust — so one invocation covers a
`dist/` holding several, and the run reports how many it checked. A root holding none of the four
fails, naming the four it looked for. `--language` is now optional: it still names the convention
for a single path, and it is how you check an already-unpacked artifact root, which carries no
extension to read.
