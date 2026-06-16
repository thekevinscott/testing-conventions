# Source-free self-test fixture

A library directory with **no language sources** — no `.py`, `.ts`, `.tsx`, `.mts`,
or `.cts`. Called with the zero-config default `["python", "typescript"]`, the
reusable workflow must detect that neither language has sources here and **skip**
every language's jobs, passing the call rather than erroring on an empty coverage
run (#94).

This mirrors our own [`packages/python`](../../../packages/python), which ships a
wheel but carries zero `.py` sources, so its coverage job has nothing to measure.
