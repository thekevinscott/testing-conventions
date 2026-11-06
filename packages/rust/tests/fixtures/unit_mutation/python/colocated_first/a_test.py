from pathlib import Path


# Sorts ahead of calc_test.py, so a full-suite run reaches it before -x stops on a kill.
def test_records_each_run():
    with Path(__file__).with_name("runs.log").open("a", encoding="utf-8") as log:
        log.write("ran\n")
