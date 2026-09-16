import sys

match sys.argv:
    case []:
        MODE = "none"
    case _:
        MODE = "some"
