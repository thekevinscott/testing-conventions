# Python — testing

**pytest, with `pytest-describe` for BDD-flavour grouping and `pytest-asyncio` for async.**

```python
# myproject/core_test.py
import pytest
from myproject.core import process

def describe_process():
    def describe_when_items_is_empty():
        def it_returns_empty_dict():
            assert process([]) == {}

    def describe_when_items_has_duplicates():
        def it_counts_each_item():
            result = process(["a", "a", "b"])
            assert result == {"a": 2, "b": 1}

    @pytest.mark.parametrize("items,expected", [
        ([], {}),
        (["a"], {"a": 1}),
        (["a", "a"], {"a": 2}),
    ])
    def it_handles_various_inputs(items, expected):
        assert process(items) == expected
```

Naming convention: **`foo.py` ↔ `foo_test.py` colocated**. The legacy `test_foo.py` prefix also works (pytest discovers both with `python_files = ["*_test.py", "test_*.py"]`).

**Why colocated**: the test file sits next to its subject. Move the source file, the test moves with it. The directory hierarchy is the source hierarchy.

**`tests/` directory** holds integration tests, e2e tests, and shared fixtures. Unit tests live next to source.

**Async tests** — `asyncio_mode = "auto"` removes the `@pytest.mark.asyncio` decorator boilerplate:

```python
async def it_awaits_the_thing():
    result = await my_async_function()
    assert result == expected
```

With `asyncio_mode = "auto"`, async tests are picked up automatically. Reserve `@pytest.mark.asyncio` for the one-off case where you need a non-default loop scope or marker.

**Fixtures** in `conftest.py`. Prefer fixtures over inline `with patch(...)`:

```python
# tests/conftest.py
import pytest
from pathlib import Path
from unittest.mock import patch

@pytest.fixture
def tmp_dir(tmp_path: Path) -> Path:
    return tmp_path

@pytest.fixture
def mock_external_api():
    with patch("myproject.external.fetch") as fetch:
        fetch.return_value = {"status": "ok"}
        yield fetch
```

For mocking a streaming external service (LLM client, network stream), build a fixture that exposes `set_response`, `set_error`, `set_responses` so each test configures the mock declaratively.

## Mocking

This section is the authoritative spec for the **"Mocking mechanism & style"** rule: the *how* of mocking. Each convention notes whether it's **machine-enforced** or **review-only**; the [enforcement summary](#enforcement-summary) collects them. Examples assume `from unittest.mock import patch`, `from typing import cast`, and `import os`.

### Patch with `unittest.mock`, never `monkeypatch`

Patch with `unittest.mock.patch` / `patch.object` / `patch.dict`, always wrapped in a `pytest.fixture` (usually `autouse=True`). **Never** use pytest's `monkeypatch` fixture, and **never** write `with patch(...)` inline in a test body.

*Why*: one patching vocabulary for the whole suite. `unittest.mock` carries the assertion surface (`assert_called_once_with`, `call_args`), composes through `patch.object` / `patch.dict`, and reads the same whether a patch is autouse or configured per-test. `monkeypatch` is a second, weaker idiom for the same job — no built-in call assertions, and it invites ad-hoc attribute pokes that drift from the `patch(...)` shape everything else uses. Inline `with patch(...)` buries setup in the test body and re-indents the assertions under it; a fixture names the mock once and hands it to the tests that want it.

Patch environment variables through the mapping — never mutate `os.environ` by hand:

```python
@pytest.fixture(autouse=True)
def mock_env():
    with patch.dict(os.environ, {"MYPROJECT_TOKEN": "test-token"}):
        yield
```

A raw `os.environ["X"] = ...` (or `del os.environ["X"]`) leaks into every later test in the process; `patch.dict` restores the prior mapping on teardown.

*Enforcement*: machine-enforced (bright-line). `monkeypatch`, inline `with patch(...)` in a test body, and raw `os.environ` mutation are each errors.

### The canonical mock

The canonical mock is an **autouse fixture wrapping `unittest.mock.patch`**, targeting the name in the *consuming* module, with defaults passed straight to `patch(...)`:

```python
@pytest.fixture(autouse=True)
def mock_is_rule_end():
    with patch(
        "gbnf.grammar_graph.get_serialized_rule_key.is_rule_end",
        return_value=False,
    ) as mock:
        yield mock
```

Use the same shape inside a `describe` block when the mock is scoped to it, and receive the fixture as a **function parameter — never `self`**:

```python
def describe_get_serialized_rule_key():
    @pytest.fixture(autouse=True)
    def mock_is_rule_end():
        with patch(
            "gbnf.grammar_graph.get_serialized_rule_key.is_rule_end",
            return_value=False,
        ) as mock:
            yield mock

    def it_stops_at_the_end(mock_is_rule_end):   # parameter, not self
        ...
        mock_is_rule_end.assert_called_once()
```

*Why — target the consuming module*: `patch` rebinds a name where it is *looked up*, not where it is *defined*. When `get_serialized_rule_key.py` does `from gbnf.is_rule_end import is_rule_end`, you patch `gbnf.grammar_graph.get_serialized_rule_key.is_rule_end` (the consumer's binding). Patch the source module instead and the consumer's already-imported reference is untouched — the mock silently does nothing.

*Why — defaults in `patch(...)`*: keeping `return_value=` / `side_effect=` on the `patch(...)` call puts the default next to its target. A test that doesn't care reads clean; a test that does overrides on the yielded `mock`.

*Enforcement*: that the patch lives in a fixture is machine-enforced (bright-line, above); the `autouse` default is convention, not pinned down. **Targeting the consuming module is review-only** — confirming it needs name resolution the checker doesn't yet do.

### Where mocks live

- **Integration mocks shared across tests** → `tests/integration/conftest.py`.
- **Unit-test mocks** → the **specific test file**, inside the `describe` block that uses them.

*Why*: scope a mock to its blast radius. The third-party double every integration test needs belongs in the integration `conftest.py`; a patch only one unit cares about belongs in that unit's `describe` block, where a reader meets it in place instead of hunting through `conftest.py`. Hoisting a unit mock up to `conftest.py` quietly makes it autouse for unrelated tests too.

### Test doubles: hand-rolled fakes over a bare `MagicMock()`

For a collaborator *object*, prefer a small hand-rolled fake class, typed at the seam with `cast(RealType, FakeThing())`. Reserve `MagicMock` for individual methods whose calls you assert. **Don't pass a bare `MagicMock()` as a whole object.**

```python
# fake collaborator — typed at the seam
class FakeGraph:
    def __init__(self, rules: dict[str, Rule]) -> None:
        self._rules = rules

    def rule(self, key: str) -> Rule:
        return self._rules[key]

def it_walks_the_graph():
    graph = cast(Graph, FakeGraph(rules={...}))   # typed fake, not MagicMock()
    assert walk(graph) == ...

# MagicMock is fine for a single asserted method
def it_flushes_once(mock_writer):
    drain(mock_writer)
    mock_writer.flush.assert_called_once()
```

*Why*: a bare `MagicMock()` answers *every* attribute and method with another mock, so the test keeps passing even after the real collaborator's interface changes out from under it — the double never complains. A hand-rolled class fails loudly when the contract drifts, and `cast(RealType, ...)` makes the type checker hold the fake to the seam it stands in for. (Convention from [thekevinscott/gbnf](https://github.com/thekevinscott/GBNF) and its `MockGraph`.)

*Enforcement*: review-only — telling a whole-object double from a single asserted method needs data-flow the checker doesn't do.

### Anti-pattern: patching module globals to inject config

**Don't patch a module-global config constant to inject configuration** — e.g. pointing `myproject.config.CACHE_DIR` at a tmp path:

```python
# WRONG — a no-op for any module that imported the value
@pytest.fixture
def cache_dir(tmp_path):
    with patch("myproject.config.CACHE_DIR", tmp_path):
        yield tmp_path
```

*Why*: a consumer that did `from myproject.config import CACHE_DIR` copied the value into its own namespace **at import time**. Patching `myproject.config.CACHE_DIR` rebinds the name in `config`, but the consumer's copy is frozen — it ignores the patch and writes to the real cache dir. (See the `config.py` shape in [setup.md](setup.md#configuration).) Inject config explicitly instead: pass the path as an argument, or take a config object at construction — one source of truth, no import-time snapshot to outrun.

*Enforcement*: machine-enforced as a **waivable warning** (`waiver:` escape hatch) — a `patch(...)` whose target is an `UPPER_CASE` module-level constant is the signal.

### What may be mocked (integration isolation)

A sibling rule about *what* you may mock — distinct from the *how* above, and enforced by the separate **"Isolation & external deps"** checker:

**In an integration test the only legal patch / mock targets are third-party** — external packages plus effectful stdlib. Any `patch(...)` whose target resolves into the library's own top-level package is an error.

```python
# OK — cachetta is a third-party dependency
with (
    patch("cachetta.write_cache", side_effect=mock_write),
    patch("cachetta.read_cache", mock_read),
):
    yield

# VIOLATION — myproject.* is first-party; an integration test must run it for real
with patch(
    "myproject.eval.evaluate.evaluate.build_iteration_cache",
    return_value=_PassthroughCache(),
) as mock:
    yield mock
```

*Why*: integration tests exist to exercise *your* orchestration end-to-end. Mocking a first-party symbol — however plausible the docstring ("we're testing orchestration, not caching") — hollows out the very wiring the test claims to cover. Drop the first-party patch and let it run; double only at the third-party boundary.

*Enforcement*: machine-enforced (bright-line) by the Isolation checker — first-party = the distribution's own package name.

### Enforcement summary

| Convention | Enforcement |
|---|---|
| No `monkeypatch` | Checker — bright-line error |
| Patches in fixtures, not inline in test bodies | Checker — bright-line error |
| Env via `patch.dict(os.environ, …)` (no raw `os.environ` mutation) | Checker — bright-line error |
| No patching module-global config constants | Checker — waivable warning |
| Hand-rolled fakes over a bare `MagicMock()` as a whole object | Review / agents only (needs data-flow) |
| Patch the name in the *consuming* module | Review / agents only (needs name resolution) |

## Coverage

`pytest-cov` with `branch=true` and `fail_under` set per project — 85 is a reasonable floor; aiming for 100 forces tests for trivia.
