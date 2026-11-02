### The Python package requires Python 3.11+

**Summary**

`testing-conventions` on PyPI declares `requires-python = ">=3.11"`. Python 3.9 and 3.10 are end
of life, and GitHub's `ubuntu-latest` image (Ubuntu 26.04) has no 3.9 build, so the package can no
longer be tested there. CI covers 3.11 through 3.13. The package reads TOML with the stdlib
`tomllib`.

**Required changes**

A consumer running on Python 3.9 or 3.10 moves its test environment to 3.11 or newer. On an older
interpreter, pip resolves to the last release that declared `>=3.9` instead of this one.

**Deprecations removed**

_None._

**Behavior changes without code changes**

_None_ on 3.11 and newer.

**Verification**

```console
$ python --version
Python 3.11.x
$ pip install testing-conventions
$ testing-conventions --version
```
