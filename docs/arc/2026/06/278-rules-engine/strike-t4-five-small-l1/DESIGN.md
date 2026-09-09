# DESIGN — five small L1s in the rete test corpus, one floor

**Status:** drawn 2026-09-08. Resolves **`4X1`**, **`4E1`**, **`4E2`**, **`4F1`**, **`4Q1`** — all
L1, all inside `tests/rete/` or `src/rete/kernel/tests/`, all small. Batched because they share a
tree and a floor, not because they share a cause.

⚠ **Every site below is a claim I took from the ledger. Six of the nine rows resolved so far had
something wrong with the row itself** — wrong count, wrong mechanism, wrong file, or not a row at
all. **Re-derive each before curing it, and report the delta.**

---

## `4X1` — the only live external suppression in 306 files, and it pleads nothing

`src/rete/kernel/tests/fanout_cost.rs:841-842` — `#[allow(unused_variables)]` with **no reason
string**. `excusare` weighed 81 exemptions across this target and this was the one defect.

**Cure — same shape as the `too_many_arguments` cluster just closed (`87285b5db`):** either the
allow carries a reason naming *which* variable is unused and why it must stay, **or the variable
goes**. Prefer removal if nothing needs it — an unused binding in a test is usually a leftover, and
`X1`'s cure removed rather than explained.

## `4E1` and `4E2` — deferral prose with nothing tracking it

- `src/rete/kernel/tests/termination_verdict.rs:5`: *"surfacing it would need a new
  `(:wat::rete::CompileOutcome)` variant behind the outcome wall, which is **affirmatively out of
  scope for the strike that split this type**."* ⛔ **"The strike that split this type" names no
  artifact a reader can find** — the file contains zero `arc`, `DESIGN` or `rune:`.
- `tests/rete/probe_arc278_P4c_native_retraction.rs:8-9`: *"the support store only buys O(delta)
  retract for a PERSISTENT cross-fire streaming engine, **a deferred surface**"*.

**Cure — and `exigere`'s own direction is the right one, keep it:** the reasoning in both reads as a
**permanent architectural invariant**, not deferred work. **Drop the deferral framing and state the
invariant.** Do not open an arc for either; do not add a tracker. ⚠ `4E2`'s primary claim (*"there is
NO cascade to build"*) is honest present-tense fact — **only the parenthetical is the defect.** Cure
the parenthetical, leave the sentence.

## `4F1` — a 460-to-10 convention with nothing holding it up

`tests/rete/probe_arc278_8i_accumulator_folds.rs:26,32,38,44,50,56,63,70,77,83` — ten bare
`.unwrap()` on `call_beside_value(...)`, against **460** sibling sites using `.expect("<context>")`.
Nothing in the type distinguishes them.

**Cure:** give the ten a context string, matching the siblings (`probe_arc278_5b_collect_rules.rs`
uses `.expect("eval")`). ⛔ **Do NOT add a lint for this** — `conformare` proposed one, and minting a
gate for a ten-site convention in one file is heavier than the defect. If you think the lint is
warranted, say so in `SCORE.md` and leave it undone.

## `4Q1` — an ambient chain with no rune anywhere in 306 files

`tests/rete/probe_arc278_import_accounting.rs:177-215`, `an_origin_already_filed_is_never_re_based`:
five steps coordinating through three `thread_local!` cells in `src/alloc_counter.rs`
(`THREAD_LIVE:91`, `SESSION_ORIGINS:168`, `LAST_ORIGIN:192`), reached by free functions whose
signatures carry none of it. `mark_session_origin_at(key, origin)` **returns nothing** — the mutation
is invisible in the type.

**Cure:** a `rune:sequi(ambient-context)` at the call site naming the invariant the comment at
`:178-180` already argues informally — that nothing else touching this thread's byte counter runs
between the reads.

⚠ **This one has a doctrine wrinkle and you must not paper over it.** The `sequi` spell says hidden
domain state *"has no rune category by design"*; this repo's `docs/CONVENTIONS.md` mints
`ambient-context` for exactly that and cites `ARM_TABLE`. **The repo's vocabulary governs here** —
`no_unknown_sequi_rune.rs` gates it — but say in `SCORE.md` that you know the spell and the repo
disagree.

---

## Out of scope = rejected

- **A lint for the `.unwrap()` convention** (see `4F1`).
- **Opening an arc for either deferral** (see `4E1`/`4E2`) — the cure is to stop calling an invariant
  a deferral.
- **`4Q1`'s deeper fix** (threading the counter explicitly) — `alloc_counter.rs` is out of this
  target and the rune is the honest target-side move.
