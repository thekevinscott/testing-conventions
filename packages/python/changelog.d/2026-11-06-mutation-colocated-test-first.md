**Changed** Python `unit mutation` runs each mutant against its colocated `<module>_test.py`
first and runs the full suite only when that test leaves the mutant alive, times out, or fails on
the clean tree. The survivor set is unchanged; a mutant the colocated test kills no longer pays
for the whole suite.
