### Node dogfood runs unit lint

**Summary**

The repository dogfood workflow now checks `packages/node/src` with `unit lint`.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

The six existing TypeScript mock factories in `packages/node/src` now use typed imports from their real modules.

**Verification**

Run `testing-conventions unit lint packages/node/src --language typescript`.
