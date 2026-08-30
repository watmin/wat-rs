# BRIEF — STONE layer-1: the seven collection impls return to `src/collection/`

Read `DESIGN-STONE-layer-1-collections-the-delegate-template.md` first.

## The work, one paragraph

Seven `#[wat_intrinsic]`-attributed functions in `src/intrinsic/collection.rs` carry their whole
implementation inline. **Move each body into `src/collection/`, and leave the attributed function
as a thin delegate that calls it.** The attribute, the doc block, the signature and the declared
arity all STAY exactly where they are; only the body's contents move. `src/intrinsic/i64.rs` is the
shape to copy — read it first.

## Read in order — the rooms, and why each

```
src/intrinsic/i64.rs:~171        `eval_i64_add` — THE TEMPLATE. A 4-line body calling
                                 `crate::runtime::eval_i64_arith`. Note what stays: the doc
                                 block, the `const OP`, the full signature.
src/intrinsic/collection.rs      the seven, with their bodies inline (see the table below)
src/collection/eval.rs           the destination for four of them. It ALREADY holds ~50
                                 `*_inner` helpers (`vector_length_inner`, `hashmap_length_inner`,
                                 `record_length_inner`, …) that these bodies already call.
src/collection/transform.rs      the destination for the other three
git show 5725ab10d -- src/collection/eval.rs src/collection/transform.rs
                                 ★ the KNOWN-GOOD BEFORE STATE. Four of the seven
                                 (`eval_rest`, `eval_vec_last`, `eval_vec_reverse`,
                                 `eval_vec_range`) lived in these files three hours ago and
                                 this diff shows them exactly as they were. For those four
                                 the move is a revert, and this is your reference.
```

## The seven and their destinations

```
  eval_length       56 ln  → src/collection/eval.rs
  eval_empty        73 ln  → src/collection/eval.rs
  eval_nth         148 ln  → src/collection/eval.rs
  eval_rest        120 ln  → src/collection/eval.rs        ← revert
  eval_vec_last      8 ln  → src/collection/transform.rs   ← revert
  eval_vec_reverse  56 ln  → src/collection/transform.rs   ← revert
  eval_vec_range    15 ln  → src/collection/transform.rs   ← revert
```

## Implementation sketch

```rust
// src/intrinsic/collection.rs — the attributed fn KEEPS everything, body becomes one call
/// … the entire existing doc block, unchanged, including @Total Unreviewed …
#[wat_intrinsic(":wat::core::length")]
pub(crate) fn eval_length(
    xs: &WatAST, list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    crate::collection::eval::length_of(xs, list_span, env, sym)
}

// src/collection/eval.rs — the body, VERBATIM, under a name of its own
pub(crate) fn length_of(
    xs: &WatAST, list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::length";
    … every line exactly as it is today …
}
```

**Naming is yours to choose** — the impl needs a name distinct from the delegate's for a reader's
sake, even though different modules would permit reuse. Say in your report what you chose and why.
For the four reverts, matching the pre-`5725ab10d` names is the obvious choice.

## Blast radius

`src/intrinsic/collection.rs`, `src/collection/eval.rs`, `src/collection/transform.rs`, and
`src/collection/mod.rs` if a `pub(crate)` export is needed. Nothing else.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **A body needs EDITING to move.** It should move verbatim. If a body cannot compile in its
   destination without a logic change, STOP and report what blocked it — that is a finding about
   the layer boundary and it is worth more than the move.
2. **You are about to move a `#[wat_intrinsic]` attribute or a doc block out of
   `src/intrinsic/`.** The verb would vanish from the completeness gate's population. STOP.
3. **You are about to widen anything beyond `pub(crate)`** to make the delegation reach. STOP and
   report the case.
4. **You are about to touch a home other than collections** — `holon/atom.rs`, `time.rs`,
   `string.rs`, any of them. Not this stone. STOP.
5. **You are about to add a lint, gate, or ledger for the delegate discipline.** Deliberately a
   later stone, derived from this one's result. STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: the seven fns, their current body line counts, and the destination you
      picked for each. Report BEFORE editing; disagree with the design's table if you disagree.
 1. ★ ALL SEVEN BODIES MOVED VERBATIM. For each, state the line count before and after and confirm
      the body text is unchanged. Any body you had to alter is STOP-1, not a delta.
 2. ★ SEVEN DELEGATES, each a single call expression. Quote all seven bodies in full in your
      report — they should be short enough that this costs you nothing.
 3. ★ THE DOC BLOCKS ARE UNTOUCHED. `git diff` on `src/intrinsic/collection.rs` must show
      deletions in fn bodies ONLY — no `///` line removed, no `@` directive moved.
 4. ★ THE COMPLETENESS GATE STILL SEES ALL SEVEN:
      cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)'
      Report its UNREVIEWED line; it must read 217, unchanged.
 5. ★ REGISTRY UNCHANGED at 429:
      grep -rhcE '^[ \t]*#\[wat_intrinsic' --include=*.rs src crates | awk '{s+=$1} END {print s}'
 6. ★ BREAK THE DOOR: pick ONE delegate, point it at the wrong impl (or delete the call), and show
      a test going red. Restore it. Quote the failure. A move that compiles proves nothing about
      whether the delegate is actually reached.
 7. ★ LINE ACCOUNTING: src/intrinsic/collection.rs before/after, and both src/collection/ files
      before/after. The registration layer must SHRINK by roughly the body mass.
 8. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 9. cargo nextest run --release -E 'test(core) + test(collection) + test(intrinsic) + test(seq)'
```

★ **Row 6 is the load-bearing one.** Rows 1–3 prove the code moved; only row 6 proves the delegate
is on the live path rather than dead alongside a duplicate.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your pre-check table. The seven before/after line counts. All seven delegate bodies, quoted. The
gate's UNREVIEWED line. The registry count. Row 6's failure, verbatim, and confirmation you
restored it. The line accounting. Then the honest deltas — above all, **anything about a body that
made the layer boundary ambiguous**, because the next stone's gate predicate gets written from
exactly that.
