**Fixed** — The Python mutation run no longer mutates test files nested below the scanned path.
cosmic-ray expands each excluded-modules entry as a glob relative to the scanned path, and the
shipped patterns (`*_test.py`, `test_*.py`, `conftest.py`) matched that path's top level alone, so
every suite in a subdirectory was mutated as production source and reported its own assertions as
survivors. The patterns are now recursive (`**/*_test.py`, `**/test_*.py`, `**/conftest.py`), so a
package with submodules is judged on its production source at every depth.
