import os

FOO = "bar"
TIMEOUT = 30 * 60
LABEL = f"{FOO}-{TIMEOUT}"
DEBUG = os.environ.get("DEBUG") or "0"
SQUARES = [n * n for n in range(4)]
