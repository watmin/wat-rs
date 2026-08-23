# BRIEF — catch-up bumps occupancy Arc, does not memcpy Vec

Hash-join first-keying holds `alpha[A]` by Arc clone.
Do not `as_ref().clone()` the occupant Vec. 7strat 3/3.
`all_left` stays. Dual-impl WHAT unchanged.
