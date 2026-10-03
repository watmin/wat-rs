# AMEND — STONE 255.87: the substrate mechanisms first, and what the conversion costs

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `6db99e46b`
(STOP-3: fifteen mechanisms, `.floor/2026-10-03T06-32-08Z`, 6181/6397). Commit locally on `main`; **do not push**.

STOP-3 did its job; the split is by kind. **This amendment is group A** (substrate defects) **and a cost measurement**.
Group B (readers and goldens: mechanisms 4, 5, 10, 12, 13, 14, 15) follows in the next amendment, after A is green.

## Group A — cure each by mechanism, its own commit, a probe that names it

- **#1 (155): `:wat::capability::Capability` is unresolved** while `wat/capability.wat` declares
  `(wat.core/defsurface wat.capability/Capability …)`. Both spellings are one identity; find where a symbol-spelled
  **declaration name** registers (or fails to) under something other than its canonical identity, and cure it at that
  door (all declaration forms, not only `defsurface`: measure `defrecord`, `defenum`, `defservice`, `defmacro`, `defn`).
  Then re-measure #9 (the negative probes' missing println), which may be downstream.
- **#2 (24): a keyword `defn` leaves `def` in expression position.** The converted `defn` macro emits `wat.core/def`; the
  freeze step that lifts declarations recognizes `:wat::core::def` only (a 5b-class keyword-only matcher the ledger did
  not see: say why it did not). Route it through the identity door.
- **#3 (9): `first` of an empty sequence at `wat/rete/compile.wat:576`** (the accumulate branch, when `acc-form` has no
  children). Trace which converted shape reaches it (the `cond-is-fact-bind` / `bind-arrow?` text compares to `":-"`
  are a lead, not a verdict) and cure the reader, not the converted text.
- **#6 (3): `disconnected`** in the journal and sqlite-store probes: trace which service died and why (its own death
  reason, not the scrubbed client view; 255.75's lesson) before curing.
- **#8 (3), #11 (1):** the closure-extraction type collector and kwargs minting decide by keyword text: through the door.

**STOP-2 stands** for any mechanism that needs a ruling; **STOP-1** for a cure that changes an unconverted program.

## The cost (measure; no cure, no timeout raised)

The floor took **546.9 s** at `50f4c0a52`, against **394–407 s** on every floor of 2026-10-03 before it (e.g.
`.floor/2026-10-03T05-24-19Z`: 394.45 s), about **+39%**. `rete::reachability` shards that ran ~29 s crossed the 30 s
kill (#7). The builder's goal for this cutover is a performance upgrade; a conversion that makes the system slower is a
finding, not noise. **Measure where the time went:** the same workload (a reachability shard, and one other heavy test)
on the unconverted and the converted stdlib, six runs each, a stated mean (`[[feedback_a_wall_clock_ratio_is_not_a_gate]]`;
FM 27); then a profile (`perf`, or the existing phase census) of the converted run naming the hot path (symbol
normalization per evaluation, identity lookups, macro expansion of converted templates: measure, do not guess). Report
it as a table. **Do not raise any time limit.** If the slowdown survives group A's cures, it gets its own stone.

## Then

The floor after group A (group B's reds are expected to remain: list the remaining mechanisms), clippy, `git status`
clean. A STOP means STOP. Append to the SCORE, commit, **do not push**.
