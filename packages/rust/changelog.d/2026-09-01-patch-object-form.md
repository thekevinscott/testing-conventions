**Fixed** `no-first-party-patch` and `no-constant-patch` read the object forms (#576). A
`patch.object(...)` / `patch.dict(...)` target is resolved through the file's imports — after
`from myproject import ledger`, `patch.object(ledger, "record")` names `myproject.ledger.record` —
and held to the same first-party and constant predicates as a string target. A base bound by no
import (a call result, a local variable, a fixture argument) resolves to no module and is left
alone. Both rules previously read only a string-literal first argument, so the object form of the
exact patch they forbid passed clean. See
[`../migrations.d/2026-09-01-patch-object-form.md`](../migrations.d/2026-09-01-patch-object-form.md).
