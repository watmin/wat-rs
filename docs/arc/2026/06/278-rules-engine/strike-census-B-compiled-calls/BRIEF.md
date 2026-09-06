# BRIEF — census B: split `compiled:calls` into `compiled:exec` + `compiled:span-elided`

Read `DESIGN.md` first, especially the C10 paragraph — a prohibition in `accum_cost.rs:85` looks
like it forbids this and does not. **No engine behaviour changes. Two string literals change, and
the gates that read them get sharper.**

## Read in order

1. `src/rete/kernel/fire/delta.rs:70-95` — the `skip_span` arm. `census_count("compiled:calls")`
   sits inside the branch that returns `Some((0u32,0u16))` **without** calling the executor. This
   becomes `"compiled:span-elided"`.
2. `src/rete/compiled_cond.rs:943-960` — the real execution site and its doc, which calls the key
   *"the compiled path's call counter"*. This becomes `"compiled:exec"`; the doc becomes true.
3. `src/rete/kernel/tests/accum_alpha_cost.rs:1330-1395` — the C4/C14 probe. Read `:1345` and
   `:1365` together: the same block asserts "bumps per pair on BOTH sides" and "counts EXECUTIONS
   only". `:1345` is the true one. The three new assertions in DESIGN replace the single
   `assert_eq!(calls_built, calls_empty)`.
4. `src/rete/kernel/tests/accum_cost.rs:40-125` — `:44` and `:93` are false; `:85-90` is C10 and
   stays; `:118` explains the NAMES list that holds the key absent on this axis — both new names
   must be absent there.
5. `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:1-40` — why a missed reader is a build
   failure and not a silent zero. This is your safety net; you do not need to add one.
6. `src/rete/kernel/fire/pass/alpha.rs:277` — the C14 note recording that `alpha:leaf-fill-pairs`
   left this key on 2026-09-03. Update it to name the split, so the history stays followable.

## Sketch

```rust
// fire/delta.rs — the elided arm names itself
let matched = if skip_span {
    census_count("compiled:span-elided");
    Some((0u32, 0u16))
} else { /* … unchanged … */ };
```

```rust
// accum_alpha_cost.rs — the probe can now tell the arms apart
assert_eq!(elided_built, exec_empty, "same (fact, alpha) pairs by two paths; …");
assert_eq!(exec_built,   0,          "the built arm must execute nothing; …");
assert_eq!(elided_empty, 0,          "the empty arm must elide nothing; …");
```

## Blast radius

`src/rete/compiled_cond.rs` (one literal + its doc) · `src/rete/kernel/fire/delta.rs` (one literal)
· `src/rete/kernel/tests/accum_alpha_cost.rs` (the probe's reads + assertions + the two contradicting
comments) · `src/rete/kernel/tests/accum_cost.rs` (two reads, two assertions, the NAMES list, the
false sentence at `:44`) · `src/rete/kernel/fire/pass/alpha.rs:277` (one history note).
**No `.wat`. No engine behaviour. No new bump site.**

## Mutation proof — required, one per site

Two bump sites, so **two mutations**, each driving the LIVE test:

1. Delete `census_count("compiled:exec")` → the probe must RED on `elided_built == exec_empty`.
2. Delete `census_count("compiled:span-elided")` → the probe must RED on the same equality, from
   the other side.

Additionally show that `exec_built == 0` and `elided_empty == 0` are not vacuous: report the actual
non-zero counts on the arms where they are non-zero, so the reader can see the equality has content.
Quote all REDs verbatim.

## STOP triggers

1. Any measured value changes beyond census key names → STOP; this is a naming strike.
2. `exec_built == 0` or `elided_empty == 0` fails → STOP and report both counts; the arms are not
   what `accum_alpha_cost.rs:1345` claims and the DESIGN is wrong.
3. You need a new bump site → STOP. Two exist; this strike adds none.
4. You find yourself editing `skip_span`'s condition → STOP. That is C10 and it stands.

## Prior result to copy for shape

`../strike-retract-removes-one/SCORE.md` — starred rows, raw evidence quoted, before/after recorded,
and the honest delta called out.
