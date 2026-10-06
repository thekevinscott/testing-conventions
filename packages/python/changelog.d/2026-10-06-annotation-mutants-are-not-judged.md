**Fixed** Python `unit mutation` drops a mutant that sits inside a type annotation — a parameter
or return annotation, an annotated assignment's type, a `type` alias's value — before the suite
runs. An annotation holds type metadata the interpreter need never evaluate (`from __future__
import annotations`, and every annotation from Python 3.14 on, per PEP 649), so cosmic-ray's
rewrite of an operator there executes no code and no test can fail on it. Such a mutant survived
a suite that pinned the function's behavior completely, failing the check on fully-tested code.
Live code beside an annotation keeps every mutant it has, a parameter default included.
