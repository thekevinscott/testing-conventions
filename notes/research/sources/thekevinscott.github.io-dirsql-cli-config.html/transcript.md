---
url: https://thekevinscott.github.io/dirsql/cli/config.html
title: Configuration File | dirsql
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

[Skip to content](#VPContent)

[dirsql](/dirsql/)

Search`K`

 Main Navigation [Getting Started](/dirsql/getting-started.html)[Guide](/dirsql/guide/tables.html)[CLI](/dirsql/cli/)[Reference](/dirsql/api/)[Migrations](/dirsql/migrations.html)[GitHub](https://github.com/thekevinscott/dirsql)

Appearance

Menu

Return to top

 Sidebar Navigation 

## CLI

[Overview & Installation](/dirsql/cli/)

[Running the Server](/dirsql/cli/server.html)

[Generating a Config (`init`)](/dirsql/cli/init.html)

[Configuration File](/dirsql/cli/config.html)

[HTTP API](/dirsql/cli/http-api.html)

On this page

# Configuration File [​](#configuration-file)

> Online: <https://thekevinscott.github.io/dirsql/cli/config>

`dirsql` can be configured with an optional config file (if omitted, server falls back to [defaults](./server.html#defaults)). Two formats are accepted:

* **`dirsql.toml`** — declarative; covers filesystem-fact tables. Works with any installation.
* **`.py` / `.js`** — native-language; lets you write `extract` callbacks in Python or JavaScript. CLI-only, and only the launcher matching the file's language can run it. See [Native-Language Configs](#native-language-configs).

## Basic Example [​](#basic-example)

toml

```
[dirsql]
ignore = ["node_modules/**", ".git/**"]

[[table]]
ddl  = "CREATE TABLE posts (_path TEXT, _basename TEXT, _size INTEGER, _mtime INTEGER)"
glob = "posts/*.md"
```

Each `posts/*.md` file produces one row in the `posts` table.

## Loading a Config File [​](#loading-a-config-file)

Pass the config file path to the `DirSQL` constructor:

PythonRustTypeScript

python

```
from dirsql import DirSQL

db = DirSQL(config="./my-project/.dirsql.toml")
await db.ready()
```

rust

```
use dirsql::DirSQL;

let db = DirSQL::builder()
    .config("./my-project/.dirsql.toml")
    .build()?;
```

typescript

```
import { DirSQL } from "dirsql";

// String argument is interpreted as a config file path.
const db = new DirSQL("./my-project/.dirsql.toml");
await db.ready;
```

By default, the root directory scanned is the config file's parent directory. Override it by passing `root` explicitly (the explicit value wins and a warning is emitted) or by declaring `[dirsql].root` in the config file itself.

## Root Directory [​](#root-directory)

By default, the config file's parent directory is the scan root. To index a different location, declare `[dirsql].root` (relative paths are resolved relative to the config file's parent):

toml

```
[dirsql]
root = "../data"
ignore = ["node_modules/**"]
```

## Stat Virtuals [​](#stat-virtuals)

Every config-defined table can expose any of these reserved columns. Add the ones you want to your DDL; the rest are silently dropped.

| Column | Type | Source |
| --- | --- | --- |
| `_path` | TEXT | The file's path relative to the scan root. |
| `_basename` | TEXT | The filename including extension. |
| `_dir` | TEXT | The parent directory path (relative to root). |
| `_ext` | TEXT | The file extension, lowercased, no leading dot. |
| `_size` | INTEGER | Size in bytes. |
| `_mtime` | INTEGER | Last-modified time, unix seconds. |
| `_ctime` | INTEGER | Created/changed time, unix seconds. |

Example query:

sql

```
SELECT _basename, _size
FROM posts
WHERE _mtime > strftime('%s', '2024-01-01')
ORDER BY _mtime DESC;
```

## Path Captures [​](#path-captures)

Use `{name}` in glob patterns to extract path segments as columns. Add a matching column name to the DDL and the capture is auto-populated:

toml

```
[[table]]
ddl  = "CREATE TABLE comments (thread_id TEXT, _basename TEXT, _mtime INTEGER)"
glob = "_comments/{thread_id}/*.jsonl"
```

A file at `_comments/abc123/2024-05-05.jsonl` produces a row with `thread_id = "abc123"`, `_basename = "2024-05-05.jsonl"`, and `_mtime` set to the file's modification time.

## Ignore Patterns [​](#ignore-patterns)

The `ignore` list skips files and directories entirely (not even scanned):

toml

```
[dirsql]
ignore = ["node_modules/**", ".git/**", "*.pyc", "__pycache__/**"]
```

The top-level `.dirsql/` directory is always excluded, whether you list it or not — it is a reserved namespace for `dirsql`'s own metadata (see [Persistence](./../guide/persistence.html)).

## Persistence [​](#persistence)

Set `persist = true` to keep the SQLite database on disk between runs instead of rebuilding from scratch on every startup:

toml

```
[dirsql]
persist = true
# persist_path = ".dirsql/cache.db"   # optional; this is the default
```

See [Persistence](./../guide/persistence.html) for the full reconcile algorithm, storage layout, and limitations.

## Loading extensions [​](#loading-extensions)

You can load SQLite extensions by specifying them in a config.

Declare each extension as a `[[dirsql.extension]]` entry:

toml

```
[[dirsql.extension]]
path       = "./ext/myext.dylib"
entrypoint = "sqlite3_myext_init"
```

* **`path`** — a path to the extension's shared library (`.so` / `.dylib` / `.dll`). Relative paths resolve against the config file's parent directory.
* **`entrypoint`** *(optional)* — the extension's init symbol. When omitted, SQLite derives a default from the filename; set it when that default does not match (for example, `sqlite-vec`'s entry point is `sqlite3_vec_init`).

**Note**: `dirsql` enables extension loading only while loading the configured libraries, then disables it again, so `load_extension()` is not exposed via SQL to the user.

Extensions add **functions** you can call in queries and in a regular table's DDL (defaults, generated columns). An extension-backed **virtual table** cannot be declared as a `[[table]]` — `dirsql` tables are per-file row tables — so a `CREATE VIRTUAL TABLE` DDL is rejected; call the extension's functions in your queries instead.

## Strict Mode [​](#strict-mode)

By default, auto-injected virtuals that aren't in the DDL are silently dropped, and undeclared user-extract keys are dropped. Enable strict mode to error when an extract emits keys not declared in the DDL:

toml

```
[[table]]
ddl  = "CREATE TABLE comments (thread_id TEXT)"
glob = "_comments/{thread_id}/*.jsonl"
strict = true
```

Strict mode does **not** apply to auto-injected stat virtuals — those are always filtered to the DDL's declared columns regardless. Strict mode applies only to keys produced by an extract callback (relevant for programmatic [tables](./../guide/tables.html)).

## Full Example [​](#full-example)

toml

```
[dirsql]
ignore = ["node_modules/**", ".git/**", "dist/**"]

[[table]]
ddl  = "CREATE TABLE comments (thread_id TEXT, _basename TEXT, _mtime INTEGER)"
glob = "_comments/{thread_id}/*.jsonl"

[[table]]
ddl  = "CREATE TABLE documents (_path TEXT, _basename TEXT, _size INTEGER)"
glob = "**/index.md"

[[table]]
ddl  = "CREATE TABLE logs (_path TEXT, _size INTEGER, _mtime INTEGER)"
glob = "logs/*.csv"
```

## Native-Language Configs [​](#native-language-configs)

You can provide a config file in a particular language, allowing you to define a dynamic extract function. This can be useful for building a database based on the *contents* of a file.

bash

```
dirsql --config dirsql.config.py
dirsql --config dirsql.config.js
```

The file looks exactly like the in-process SDK construction — same `DirSQL` / `Table` API:

dirsql.config.pydirsql.config.mjsdirsql.config.cjs

python

```
import json
from dirsql import DirSQL, Table

def extract_meta(path):
    with open(path) as f:
        return [json.load(f)]

# Python must export an `app` variable
app = DirSQL(
    tables=[
        Table(
            ddl="CREATE TABLE papers (title TEXT, _path TEXT)",
            glob="**/meta.json",
            extract=extract_meta,
        ),
    ],
)
```

javascript

```
import { readFileSync } from "node:fs";
import { DirSQL } from "dirsql";

export default new DirSQL({
  tables: [
    {
      ddl: "CREATE TABLE papers (title TEXT, _path TEXT)",
      glob: "**/meta.json",
      extract: (path) => [JSON.parse(readFileSync(path, "utf8"))],
    },
  ],
});
```

javascript

```
const { readFileSync } = require("node:fs");
const { DirSQL } = require("dirsql");

module.exports = new DirSQL({
  tables: [
    {
      ddl: "CREATE TABLE papers (title TEXT, _path TEXT)",
      glob: "**/meta.json",
      extract: (path) => [JSON.parse(readFileSync(path, "utf8"))],
    },
  ],
});
```

Only the extension matters — the file can be named anything. `dirsql.config.{py,mjs,cjs}` is the suggested convention but not required.

### Module conventions [​](#module-conventions)

* **Python (`.py`)** — module-level `app = DirSQL(...)`.
* **ESM (`.mjs`, or `.js` in an ESM package)** — `export default new DirSQL(...)`.
* **CommonJS (`.cjs`, or `.js` in a CJS package)** — `module.exports = new DirSQL(...)`.

Pager

[Previous pageGenerating a Config (`init`)](/dirsql/cli/init.html)

[Next pageHTTP API](/dirsql/cli/http-api.html)

Released under the MIT License.

Copyright 2024-present
