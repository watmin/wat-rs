# BRIEF — catch-up BORROWS the parent beta, it does not take it

`wm.beta.remove` + two re-inserts becomes `wm.beta.get`. Every
mutable touch in that window is `bind_pool` / `match_pool`,
disjoint fields from `beta`, so the borrow checker never needed
the parent out. The hand-held restore invariant disappears with
it. Differential is the gate. Do not add a guard — there is
nothing left to guard.
