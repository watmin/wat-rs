# DESIGN — a phase mark nobody emits reads as 0.00 ms and is then subtracted

## Why

Work-list **C3**. `accum_cost.rs:1603` (⚠ the row said `:1630`) reads the census row
`"  │  setup:seen:insert"`. The reader is:

```rust
let of = |name: &str| -> u64 { rows.iter().find(|(nm, _, _)| *nm == name)
                                   .map(|(_, ns, _)| *ns).unwrap_or(0) };
```

**`unwrap_or(0)` turns "this mark does not exist" into "this mark measured zero."** The table prints
`insert 0.00 ms` as a measurement and then derives `in-fire insert − S`, which is `0 − S` — the
whole isolated cost, negated, presented as a difference. Driven at HEAD:

```
in-fire
setup:seen                       0.00 ms
alloc                          0.00 ms
insert                         0.00 ms
S  seen_insert loop              2.55 ms
in-fire insert − S              -2.55 ms
in-fire seen − S                -2.54 ms
```

⚠ **The rows above are FLUSH-LEFT, and an earlier draft of this file indented them.** The `\`-newline
continuation in the `format!` eats the leading whitespace (work-list **C11**), so the indent never
reaches stdout. I re-typed the block with the indent it *should* have had and presented it as driven
output; a rider nearly chased a regression that never existed. ⚠ `2.55` is box-specific — observed
1.79–4.28 across runs. The stable claim is the **identity**: `in-fire insert − S` was always exactly
`−S`.

## ⛔ THE ROW UNDERSTATES IT: ALL THREE IN-FIRE ROWS ARE ZERO, AND THE OTHER TWO MARKS EXIST

`setup:seen` and `setup:seen:alloc` are both in `REQUIRED_PHASES` and both fire. They read 0.00
anyway, and `fire/delta.rs:270-276` says why:

```rust
let __seen = phase_start();
let __seen_alloc = phase_start();
let mut seen_ids: FxHashSet<u64> = FxHashSet::with_capacity_and_hasher(...);
let mut seen_rest: FxHashSet<Value> = FxHashSet::default();
phase_end("  │  setup:seen:alloc", __seen_alloc);
phase_end("  ├ setup:seen", __seen);
```

**The two marks wrap the SAME region — two allocations — and nothing else.** `setup:seen` is not a
parent containing an alloc half and an insert half; it is *identical in extent* to
`setup:seen:alloc`. So `setup:seen:insert` is not a typo for an existing mark: **it names work that
is not inside this phase at all.**

The insert cost is real and already counted elsewhere: `seen_insert` is called from
`fire/pass/alpha.rs:58` and `fire/pass/production.rs:114`, inside the `alpha` and `production`
phases. There is nothing unmarked to name.

So the table's premise — *"in-fire `setup:seen`, split alloc vs insert"* — is false at the root. Its
own doc comment (`:1528`) states that premise, and `DESIGN-STONE-seen-fire-context` is cited for it.

## The class, MEASURED before deciding the rung

35 `unwrap_or(0)` mark-readers across 12 cost-test files. Every one converts an absent mark to zero.
But the corpus was swept for names read-but-never-emitted (phase marks *and* census counters, 49 + 26
engine-side literals):

> **Exactly ONE** test-side name resolves to nothing: `"  │  setup:seen:insert"`, `accum_cost.rs:1603`.

⛔⛔ **THAT MEASUREMENT WAS FALSE. THE ANSWER IS FIVE** — corrected 2026-09-02 after the strike ran;
see `SCORE.md` § A. My sweep used a naive `"…"` regex with no notion of comments or char literals,
so one unbalanced quote inverted the parity and it matched the *gaps between* literals. The four
`ALPHA_KIDS` entries in `accum_cost.rs` are read through a loop variable and were invisible to it.
**They are a different disposition** — that reader branches on `kid_pairs == 0` and never lets the
nanoseconds answer whether the mark exists — so they are justified in place with
`rune:lint(census-name-retired)`, not deleted. The rung below is unchanged by the correction: five
instances, one of them a real defect, still does not license a 35-site migration.

That measurement decides the rung. Migrating 35 sites to a helper that cannot return 0 dies at
**Simple** — 35 sites and 12 files for one live defect (see the correction above). A **construction-time check** survives all
four questions and is the idiom already in this tree (`rete_citation_resolves`,
`rete_names_in_wat_scripts_resolve`).

## The contract decision, pinned

**A phase-mark name read by a cost test must resolve to a `phase_end` / `census_count` literal in
non-test `src/`, or the lint is RED.** Absence becomes a build failure instead of a zero.

And the table is restated to what is true: `setup:seen` is allocation-only and coextensive with
`setup:seen:alloc`; the insert cost lives in `alpha` and `production`.

## Out of scope = REJECTED

- Adding a `setup:seen:insert` mark to the engine. There is no unmarked region — the work is inside
  two phases that already time it, and a new mark would double-count.
- Migrating the other 34 `unwrap_or(0)` sites. Measured: zero live instances. The lint covers them
  from now on, which is what a check at construction is *for*.
- C10 (`compiled:calls` is a designed union) and C11 (the indent slip). Separate rows.
