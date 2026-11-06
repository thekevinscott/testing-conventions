**Changed** `unit mutation --language python --base` hands the Python adapter the changed line
ranges as `--lines <file>:<start>-<end>`, so a mutant off the diff never runs the suite. Every
mutant in a touched module used to run and was dropped only afterwards. Reported results are
unchanged.
