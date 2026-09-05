**Changed** — A Python surviving mutant now names the source the mutation produced, the way a
TypeScript one already does: `calc.py:2: core/ReplaceBinaryOperator_Add_RShift (-> return a >> b)`
rather than the cosmic-ray operator class alone. The bundled adapter reads the mutated line out of
the work result's own diff and reports it as the mutant's replacement; a mutation whose diff adds
no line (a removal) is still listed, by its operator name.
