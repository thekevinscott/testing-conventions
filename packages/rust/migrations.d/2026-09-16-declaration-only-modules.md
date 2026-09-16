### Declaration-only modules are not colocated-test subjects

**Summary**

`unit colocated-test` flagged every Python file with code and every TypeScript file with runtime
code, so a re-export barrel or a constants module needed a test that restated the source, or an
exemption whose reason said "barrel". The subject predicate is now "holds a function or control
flow anywhere in the file": a declaration-only module is skipped by the parser, the line the Rust
arm already drew. Presence and co-change share the predicate, so both move together.

**Required changes**

_None._ Exemptions that named a barrel or a constants module can be deleted:

```toml
# Before
[[python.exempt]]
path = "mypkg/__init__.py"
rules = ["colocated-test"]
reason = "re-export barrel; no logic of its own"

# After: the entry is removed
```

An entry left in place still loads (its path exists); it lifts nothing.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python or TypeScript file with no function and no control flow no longer appears in the presence
report, and editing one no longer requires a co-change to its test. A file with `def`, `async def`,
`lambda`, a method, `if`, `for`, `while`, `try`, `with`, `match`, a conditional expression, or a
comprehension with an `if` — in TypeScript a function declaration or expression, an arrow, a method
with a body, `if`, `for`, `while`, `do`, `switch`, `try`, or a ternary — is a subject exactly as
before. `and`, `or`, `||`, `&&`, and `??` do not make a file a subject. Rust is unchanged.

**Verification**

```console
$ testing-conventions unit colocated-test --language python src
```

A barrel `__init__.py` or a constants module with no `_test.py` twin is absent from the output; a
module with a `def` and no twin is still listed.
