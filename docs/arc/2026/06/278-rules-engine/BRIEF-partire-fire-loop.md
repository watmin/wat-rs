# BRIEF — partire the fire loop

`fire_fixpoint_delta_armed` is 1774 lines, 12 levels deep, 9 passes.
Extract each pass along the `// ── N.` seams it already carries,
into `fire/pass/`. ONE pass per commit, differential green each
time. Behaviour byte-identical — this is a move, not a rewrite.
Do not fix a clone, a name or a comment on the way through.
