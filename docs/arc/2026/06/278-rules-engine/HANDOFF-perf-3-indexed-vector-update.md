# HANDOFF — perf 3: the indexed vector update

The store's writes cost O(table): **4.9 ms per put** at 1000 rows and climbing (3.33× → 3.67× per
doubling), against 0.6 ms per scan flat after perf-2. `put` is a nested foldl over the whole table
(`mem.wat:516-531`); `delete` walks it the same way.

Start here, in order:

1. `DESIGN-STONE-the-indexed-vector-update.md` — why the defect is in **core**, not the store, and
   the two routes rejected.
2. `BRIEF-perf-3-indexed-vector-update.md` — the rooms as exact `file:line`, four STOP triggers.
3. `SCORE-perf-2-store-read-path.md` — the stone before this; its Row 3 section is why this brief
   refuses to predict the circuit's number.

Three things to hold:

**The store is not the defect.** `PersistentVector` is `rpds` (`Cargo.toml:123`) — indexed `set` is
O(log n) — but **wat exposes no indexed update at all**, only `stream->pvec`. A keyed write has no
choice but to degrade to a fold when the language cannot address a slot. So expose the primitive,
then let the store use it. You are exposing what already exists, not implementing a structure.

**★ Order-independence is what makes the store side cheap, and it is a claim to VERIFY.** After
perf-2 no read path touches `Record/rows` — only the `:init` rebuild, `put`, and `delete` — so the
table is an unordered bag and `delete` can swap-remove. **Check that yourself before relying on it.**
If any site reads it in order, swap-remove is unsound and the stone changes shape.

**Swap-remove is the change most likely to trip the differentials**, because it is the one that moves
a row. That is the check working, not bad luck: if one goes red, behaviour moved — STOP and report
which, and never adjust the test.

Report the circuit's wall time; do not promise one. The stone before this predicted a circuit
improvement from a read-only measurement and was wrong — that is one stone old and it is the reason
row 3 says *measured, not predicted*.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-perf-3-indexed-vector-update.md` when done. It will be graded by re-running.
