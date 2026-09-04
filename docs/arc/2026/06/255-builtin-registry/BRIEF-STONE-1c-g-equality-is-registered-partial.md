# BRIEF — STONE 1c-g: `=` and `not=` are registered `Partial`

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/` is
harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-1c-g-equality-is-registered-partial.md` (sibling) first — it carries the measured
blast radius and the reasoning you should not re-derive.

## The work in one paragraph

`:wat::core::=` and `:wat::core::not=` are `@Totality Partial`, proven by a committed counterexample.
Register both, then delete every place that asserted otherwise: a by-name `matches!` placeholder, two
entries in a purity residue, two in an expand-time residue, and the gate parser that reads the
placeholder's shape. Four fixtures then correctly go red because they rest on the old lie; bring
each to the truth.

## Rooms, in order

1. **`[[NOTE-equality-is-argued-proven-partial-and-held]]`** — the two complete doc blocks, verbatim,
   with their `#[wat_intrinsic]` wrappers. **This is a transcription job. Do not re-argue any axis.**
   Both wrappers forward to existing fns and change nothing about them.
2. **`src/runtime.rs:5228` (`eval_eq`) and `:5274` (`eval_not_eq`)** — where the wrappers land. Both
   take `head` as their first parameter, so neither fits the canonical `#[wat_intrinsic]` shape and
   neither may be reshaped; the NOTE's wrappers exist for exactly that reason. Leave both untouched.
3. **`src/rete/purity.rs:526-527`** — `":wat::core::="` and `":wat::core::not="` in the `pure_det`
   list. Delete both. Leave a departure marker in the house style already used at that site for
   `reduce` / `str` / `u8` / `do` / the comparison ops.
4. **`src/rete/purity.rs:653-660`** — the `matches!` placeholder. Both names leave, which empties it.
   **Delete the whole `Some(Unreviewed) | None => matches!(…)` arm and replace it with `=> false`.**
   Its own header has said since it was written that a homed name must leave it; both are homed now,
   so the placeholder has no reason to exist. Rewrite the long comment block above the `match` that
   explains the per-verb hand-list reasoning — it is describing a mechanism that is gone.
5. **`src/intrinsic/mod.rs:3360`** — the residue gate's parser for that `matches!` block. Measured:
   with the block gone it panics `"the total derivation's fallback matches! not found in
   intrinsic_meta — has it moved/renamed?"`. The gate was built to police a hand-list that no longer
   exists; retire that half of it. ⛔ **Keep everything else in that test** — the `pure_det` residue
   parse, the `is_expand_time_legal` residue parse, both non-emptiness assertions, both named
   anchors, and the real assertion that no residue name resolves in the registry.
6. **`src/macros/eval.rs:491-492`** — `=`/`not=` in the expand-time residue hand-list. Both doc blocks
   declare `@ExpandTime Legal`, so registering REPLACES these entries; deleting them without the
   registration would silently revoke today's legality. Delete both. The residue gate's other named
   anchor cites `:wat::core::List?` at `src/macros/eval.rs:492` — that line moves; fix the citation.
7. **The four fixtures.** Bring each to the truth; the DESIGN's table says which repair each takes.
   - `tests/services/probe_arc278_sift_logs.wat` — an enum comparison. Repoint to
     `:wat::rete::core::enum::=`.
   - `tests/services/probe_arc278_sift_arena.wat` — mixed. The `(= shp 0)` sites take
     `:wat::i64::=`; the `(= (ForeignRecord/class fr) "prod::Alert")` site takes
     `:wat::rete::string::=`; the `(= s "high")` site compares a `Value` and **has no typed
     replacement** — see below.
   - `tests/rete/probe_arc278_foreign_pred_purity.{wat,rs}` — its `foreign_pred_is_total` case
     asserts a `Value`-comparing predicate is total. **That assertion is now false.** Invert it into
     a negative witness: the predicate is NOT total, and say in the test's own doc why —
     `ForeignRecord/get` yields `(Option Value)`, `Value`'s declared domain admits `Fn`,
     `values_equal` has no `Fn` arm. Its `pure` and `deterministic` siblings are unaffected; leave
     them.
   - For any sift case whose predicate compares a `Value`, sift's fence now refuses it and the
     service returns `::Fatal` with a Fault rather than survivors. Change the test to assert **that**
     — the refusal, with its Fault — not a survivor count. Do not weaken the fence and do not delete
     the test: the refusal is the behaviour under test now.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if any axis in the NOTE's doc blocks does not transcribe cleanly (a cited `file:line`
  has moved, a fn was renamed), STOP and report which. Do not re-argue the axis and do not
  substitute your own grade.
- **STOP-2** — if removing the `matches!` arm turns any test red **other than** the five named in the
  DESIGN's measured blast radius, STOP and report it verbatim. Run this census yourself rather than
  trusting my list: `cargo nextest run --release 2>&1 | grep -E "^\s+FAIL"` is the orchestrator's to
  run, so instead report the scoped results for `binary_id(wat::rete)`, `binary_id(wat::services)`,
  and `test(the_residues_cannot_shadow_the_registry)` and let the orchestrator reconcile.
- **STOP-3** — do not weaken sift's fence, do not add a special case for `=`, and do not delete any
  fixture. A predicate that is genuinely partial must be refused; the tests move to asserting the
  refusal.
- **STOP-4** — if `cargo build --release` succeeds but `probe-core-eq-is-partial.wat` or
  `probe-eq-generic-instantiation.wat` in `wat-scripts/scratch-pad/` changes behaviour, STOP and
  report — those are the committed counterexamples this grade rests on.

## What to report

The two registrations' final doc-block headers (the five `@` axis lines each); the scoped results
for `wat::rete`, `wat::services`, `wat::collection`, and the residue gate; the exact disposition of
each of the four fixtures with its diff; and anything that surprised you.
