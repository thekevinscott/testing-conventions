# Isolate tests

Isolation is about which collaborators a test may fake — and the answer is *opposite* for the
two kinds of test:

- A **unit test** isolates one unit: everything it depends on — first-party collaborators *and*
  external packages / Node built-ins — is mocked, so a failure points at the unit, not a
  collaborator.
- An **integration test** isolates the *system*: first-party code runs for real, and only the
  outside world — third-party packages and Node built-ins (databases, the network, the clock,
  the filesystem) — is mocked.

Both rules turn on the same bright line: **first-party vs. external**. `testing-conventions`
draws it deterministically, with no module resolution:

| Specifier                          | Origin        | Example                    |
| ---------------------------------- | ------------- | -------------------------- |
| relative / absolute path           | first-party   | `./service`, `../core`     |
| `node:`-prefixed or a builtin name | Node built-in | `node:fs`, `child_process` |
| any other bare specifier           | third-party   | `stripe`, `@scope/pkg`     |

## Don't mock first-party code in an integration test

This direction is enforced for TypeScript today by the `no-first-party-mock` lint. Point
`integration lint` at your integration suite:

```sh
testing-conventions integration lint --language typescript test/integration/
```

It parses every `*.test.{ts,tsx,mts,cts}` file and flags any `vi.mock()` / `vi.doMock()` of a
first-party module:

```ts
// ❌ flagged — ../src/ledger is first-party; an integration test must run it for real
vi.mock('../src/ledger', () => ({ record: vi.fn() }));

// ✅ fine — a third-party package and a Node built-in
vi.mock('stripe');
vi.mock('node:fs');
```

A first-party mock prints to stderr and the command exits non-zero, so CI fails. A non-literal
target (`vi.mock(name)`) can't be classified deterministically and is left alone.

## Mock every collaborator in a unit test

The mirror-image rule for the **unit** suite: mock each first-party collaborator and external
import, typed so the double can't drift from the real module.

```ts
vi.mock('./service', async () => {
  const actual = await vi.importActual<typeof import('./service')>('./service');
  return { ...actual, fetchUser: vi.fn() };
});
```

See the [README](https://github.com/thekevinscott/testing-conventions#isolation) for the full
rule across Python, TypeScript, and Rust. (Deterministic enforcement of the unit direction is
rolling out alongside `no-first-party-mock`.)

## See also

- [Reference — `integration lint`](../reference/#integration-lint) — the lint list and exit codes.
- [Exempt a file](./exemptions) — the reason-required escape hatch, shared by every rule.
