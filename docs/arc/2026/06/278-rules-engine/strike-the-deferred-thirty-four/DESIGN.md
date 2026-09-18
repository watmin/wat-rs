# DESIGN — 34 dead pointers, and the basename that resolves is the trap

## Why

F2-e's gate (`5aa25e0c4`) widened its scan to `wat/` and `wat-tests/` and found **34 stale path
citations**, fenced in a `DEFERRED` const rather than fixed — STOP-1, correctly. **The class is
closed** (a new stale path in those files reds today); this is the backlog behind it.

## Measured, not inherited

Parsed from the `DEFERRED` const at HEAD, each cited basename searched tree-wide:

| | count |
|---|---|
| **a file of that basename exists elsewhere** | **12** (11 distinct targets) |
| **no file of that name anywhere** | **22** |

⚠ **A first parse said 14/28 and was wrong** — the regex matched tuple literals anywhere in the file,
including the gate's own seven extractor unit tests, so `"rs"`, `"//"` and `";;"` were counted as
cited paths. Scoping to the const gives 34 exactly. **Assert the row count before trusting the split.**

## ⛔ THE TRAP: A BASENAME THAT RESOLVES IS A HINT, NOT A VERDICT

`wat/rete.wat` cites **`kernel/tests.rs`**. Basename search resolves it to **`src/macros/tests.rs`** —
and that is almost certainly **wrong**. The rete kernel's tests were `src/rete/kernel/tests.rs`,
deleted 2026-08-20; `src/macros/tests.rs` merely shares a filename.

**Re-pointing on a basename match would replace a dead pointer with a confident wrong one**, which is
worse: a dead link announces itself, a plausible one does not. Every re-point needs the target
verified as *the same thing the sentence is about*, not merely a file with that name.

`wat-scripts/lib/gen.wat` resolves to **two** candidates and needs the same treatment.

## The contract decision, pinned

**Re-point what is verifiably the same artifact; DELETE the claim where the artifact is gone.**

- **Moved (≤12):** re-point **only** where the target is confirmed to be the same thing — by content,
  not by name. Where it cannot be confirmed, treat it as gone.
- **Gone (≥22):** **delete the path**, keeping whatever the sentence says that is still true. A
  citation to a deleted file is a dead pointer; F0's *"deleting the claim is the fix"* applies
  directly.
- **Prefer a symbol to a path where the sentence allows it.** A symbol survives a move; a path does
  not. C14 lost two citations to line drift and F2-e lost six to a file split.

## ⛔ A COMMENT-ONLY EDIT TO THE STDLIB IS NOT FREE

F2-e's rider turned the floor RED with a comment cure: **four golden EDNs hard-pin absolute
`wat/core.wat` line numbers**, and a 2-line comment becoming 3 shifted them.

Any edit to a file whose lines are pinned by a golden **must be line-count-neutral**, or the goldens
must be regenerated deliberately. `wat/core.wat` is known-pinned; **the strike must determine which
other files are** rather than assume it is the only one.

## Out of scope = REJECTED

- **Widening the gate further.** Its scope and depth are settled; this is backlog.
- **Fixing citations outside the `DEFERRED` const.** The const is the population. If the walk finds
  new ones, that is a finding, not scope creep to absorb.
- **Deleting the `DEFERRED` const before the list is empty.** It shrinks to zero, then goes.
