### The vitest base declares the text coverage reporter

**Summary**

`vitestConfig` now declares `coverage.reporter: [['text', { skipFull: false }]]`. Vitest 4
injects `skipFull: true` into the text reporter when an agent environment variable is present,
and at the 100% floor that omits every row from the coverage table. Vitest spreads declared
options over the injected flag, so the declared value wins; the base owning the entry fixes
every consumer at once.

**Required changes**

`mergeConfig` concatenates reporter lists, so a consumer's `coverage.reporter` now appends to
the base's `text` entry. Declare only the reporters you add, and drop a re-declared `text` —
vitest keeps duplicate entries and prints the table once per entry.

```ts
// before
reporter: [['text', { skipFull: false }], 'json-summary'],
// after
reporter: ['json-summary'],
```

**Deprecations removed**

_None._

**Behavior changes without code changes**

A consumer that declares no `coverage.reporter` moves from vitest's default list (`text`,
`html`, `clover`, `json`) to the base's `text` alone: the table still prints, and `coverage/`
holds the reports you declare. To keep any of the file reporters, name them:
`reporter: ['html', 'clover', 'json']`. The coverage gate itself is unchanged — it names its
reporter on the CLI (`--coverage.reporter=json-summary`), which replaces the declared list.

**Verification**

With an agent environment variable set, run coverage:

```
CLAUDECODE=1 npx vitest run --coverage
```

The text table lists every file, and the `All files` row reads 100 in each column.
