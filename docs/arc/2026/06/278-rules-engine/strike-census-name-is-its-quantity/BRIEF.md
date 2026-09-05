# BRIEF — make `filter:test-pass` name one quantity

**Floor GREEN when you are done.** Census numbers WILL move — that is the deliverable, not a STOP.

## Read in order

1. **`DESIGN.md`** — and note the cut: twelve other census sections are rowed and out of scope.
2. **`src/rete/kernel/fire/mod.rs:2063-2105`** — `dispatch_where_tests`. The three `filter:test-pass`
   increments and the `filter:test-reuse` / `filter:test-evals` sites around them.
3. **`src/rete/kernel/tests/node_share_cost.rs:290-300`** — the axis precondition
   (`fire_reuse > 0 && fire_evals == 0`), which is what makes the rest vacuous.
4. **`node_share_cost.rs:860-900`** — `wasted`, `waste_pct`, and the three assertions.
5. **`src/rete/kernel/tests/right_index_counter_invariant.rs`** — `assert_applicable`: **the house
   form for a probe refusing an axis it cannot decide**, with a control test that drives the refusal.
6. **`docs/.../vigilia-2026-09-05/recon/census-name-audit.md` § A** — the finding as recorded.

## The work

**1. Split the union.** The reuse arm at `:2068` stops bumping `filter:test-pass`. Give the
tree-proven push its own key if a consumer needs it (`filter:test-reuse` at `:2067` may already be
enough — say which, and why). Add a one-line doc at **every** increment site naming the quantity.

**2. Make the three assertions honest.** With the union split, `wasted = evals - passes` is
arithmetic on this axis only if `evals > 0`. It is not. So either:
   - drive the waste gate on an axis that uses the eval arm, or
   - have it **refuse** this axis explicitly, in the `assert_applicable` shape, with a control test
     that drives the refusal.

   **A gate that reports `0.0 < 50.0` as a measurement must not survive this strike in any form.**

**3. Say what moved.** Every recorded census number that changes, with its before and after.

## Blast radius

`src/rete/kernel/fire/mod.rs` + `src/rete/kernel/tests/node_share_cost.rs`, plus any test reading
`filter:test-pass`. **No engine behaviour change** — same facts, same rows.

## STOP triggers

1. **If splitting the key changes any FACT-level result, STOP.** Census is `#[cfg(test)]`; a
   behaviour change means the split touched the engine.
2. **If another test's assertion depends on the union meaning, STOP and report it** — that is a
   second consumer of the mis-named quantity and a finding in its own right.
3. **If you find yourself adjusting a threshold to keep a gate green, STOP.** The DESIGN forbids it:
   the number is not wrong, the number is not a number.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`right_index_counter_invariant.rs` — an applicability guard extracted as a verb so a control test can
DRIVE its refusal rather than describe it.
