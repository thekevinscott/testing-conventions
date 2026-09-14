### Python `unit coverage` scopes the denominator to the scanned path

**Summary**

Python `unit coverage` measured the source of a package's editable path dependencies alongside the
package's own source. An editable install resolves through a `.pth` to the real source tree, which
is not under `site-packages`, so coverage.py's default "measure everything that isn't stdlib or
third-party" picked it up — a package whose own tree was at 100% failed on code that belongs to a
different package, covered by that package's own suite. The run now passes `--source` (the scanned
path), so only the scanned path's sources are the denominator, matching the TypeScript arm.

**Required changes**

_None._ The change is inside the Python coverage run; a `uses:` call or direct
`testing-conventions unit coverage --language python <source>` needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python package with an editable path dependency whose tree sat outside the scanned `source` no
longer has that dependency's coverage counted in its floor. A package that was red solely because
an editable dep dragged the total below the floor now passes; a package whose own floor was
previously lowered to paper over the dep should restore the floor to its real value (the dep no
longer affects it). The whole-tree floor now also counts a source file under the scanned path that
no test imports as 0% covered — matching the TypeScript arm — so an untested source file that
previously vanished from the report now fails the floor.

**Verification**

Run the gate on a Python package whose suite imports an editable path dependency:

```console
$ testing-conventions unit coverage --language python src
```

The report lists only files under the scanned `source`; the editable dependency's files no longer
appear, and a package whose own tree meets the floor passes.
