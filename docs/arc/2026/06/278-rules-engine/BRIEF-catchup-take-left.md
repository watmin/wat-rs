# BRIEF — catch-up takes parent beta, does not clone Vec

Hash-join first-keying `remove`s `beta[P]`, walks it,
inserts it back. Do not `.cloned()`. 7strat 3/3.
Do not Arc-wrap BetaMemory. Dual-impl WHAT unchanged.
