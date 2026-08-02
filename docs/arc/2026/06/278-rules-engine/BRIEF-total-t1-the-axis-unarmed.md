# BRIEF — T1: the third axis, UNARMED, and the worklist it enumerates

Design: `DESIGN-STONE-total-the-third-axis.md` — read the body AND the `✦ STATUS UPDATE` at the end.
The update carries four things that post-date the body; do not act on the body alone.

## The work, in one paragraph

A rete `where` predicate must be pure ∧ deterministic. `first`/`second`/`third`/`nth` and
`i64::/`/`mod`/`rem`/`quot` are all pure, all deterministic, and all **partial** — undefined on some
inputs — so the fence admits every one of them. `first`-on-empty compiles, fires correctly until a
rule meets an empty vector, and then aborts the entire fire. Add **totality as a third axis** beside
Pure and Deterministic, register `:wat::rete::total?` beside its two siblings, and then **measure**:
run it over every `where` expression in the 98-row corpus and report exactly which verbs a live row
demands be classified. **Do NOT wire it into the fence.** The fence stays two-conjunct in this strike.

## Why unarmed — this is the load-bearing constraint, not caution

A refused `first` with nowhere to go locks a user out of arithmetic inside a `where`. The total
variants (`:undefined`-carrying siblings) do not exist yet, so arming now would ship a refusal before
its destination exists. The order is **enumerate → mint → migrate → arm**, and you are the first step.

Your enumeration IS the deliverable. It is what makes the mint honest: the corpus names which verbs
actually need a total sibling, instead of someone guessing a list.

## Rooms — read in order

1. `src/rete/purity.rs` module doc — the two-axis model, DEFAULT-DENY, the arc-255 successor note.
2. `src/rete/purity.rs` — `enum Axis` (now `pub(crate)`), `OpMeta` (`pure`, `deterministic`),
   `intrinsic_meta` and its 7 `OpMeta{}` construction sites, and the ~110-verb `matches!` arm that
   feeds one of them.
3. `src/rete/purity.rs` — `classify_expr` / `head_ok` / `classify_fn`, now
   `Result<(), AxisViolation>`; `is_pure_expr` / `is_deterministic_expr` deriving via `.is_ok()`;
   `find_axis_violation` / `eval_axis_violation`.
4. `wat/rete.wat` — `(defenum :wat::rete::Axis …)`, `AxisViolation`, and **`axis-violation-message`'s
   exhaustive `match`** (the thing your new variant will break — see the STOP below).
5. `src/check.rs` — where `:wat::rete::pure?` / `deterministic?` are registered. A third sibling
   registers beside them. **Confirm the line numbers yourself; they drift in this arc.**

## The shape

Mirror the two axes that exist. Nothing here is novel — `Total` is a third value of an enum whose walk
is already axis-generic.

- `Axis::Total` beside `Pure` / `Deterministic`.
- `OpMeta.total`, set at the 7 construction sites.
- `is_total_expr` deriving via `.is_ok()`, exactly like its two siblings.
- `eval_total_predicate` + registration of `:wat::rete::total?` (`-> :bool`, same signature shape).
- The new arm in `axis-violation-message`'s exhaustive match.

**DEFAULT-DENY, and do not soften it.** Every verb is `total: false` until a live corpus row demands
otherwise. Do **NOT** mass-assert `total: true` across the ~110-verb `matches!` list — those were
vetted for a *different* property, and carrying the claim across is precisely the hand-audit stem that
file's own doc condemns. Classify only what your measurement names, and say why for each one you do.

## The measurement — the actual deliverable

After the axis exists and builds, run `total?` over every `where` expression in the corpus
(`wat-scripts/perf/grid/where-*.wat`, 9 files / 98 rows) and report:

- **Which verbs** appear in a `where` and are currently not-total. Deduplicated, with a count of rows
  each appears in.
- **Which rows** would be refused if the fence were armed today, by name.
- Your read on which of those verbs are *genuinely partial* (want an `:undefined` sibling in T2) versus
  *total but unclassified* (want a `total: true` classification and no sibling). This is a judgment
  call and I want it marked as one — flag anything you are unsure of rather than deciding it.

How you run the measurement is yours to choose; a throwaway probe is fine. If you leave a `.wat`
artifact behind it must live under `tests/` or be deleted — **not** `wat-scripts/`, where every file is
loaded and type-checked by a corpus gate.

## STOP triggers — rejection criteria. Ship nothing, report.

- **⛔ STOP-1 — the `_` wildcard.** Adding `Axis::Total` will break `axis-violation-message`'s
  exhaustive match. **That is the mechanism working, not an obstacle.** `check.rs`'s own error text
  offers `"(or include `_` wildcard)"` as a way out. **Taking it is a rejected strike.** The `_`-arm
  ban on an enum scrutinee is doctrine (`109/NOTE-full-enum-match-mandatory-no-wildcard-arm.md`) whose
  checker rule is still deferred, so nothing will stop you — which is exactly why this is a STOP. Name
  every variant.
- **STOP-2 — the fence changes behaviour.** `total?` is callable but MUST NOT be consulted by
  `compile-condition`. If the accepted-`where` set moves by one row, halt. The 98-row corpus gate is
  how you prove it didn't.
- **STOP-3 — `pure?` / `deterministic?` verdicts move.** They must be byte-identical. Adding a field to
  `OpMeta` must not perturb the other two axes' answers.
- **STOP-4 — the measurement is vacuous.** If `total?` returns false for *everything* including verbs
  no corpus row uses, or true for everything, the classification is not discriminating and the
  enumeration is worthless. Sanity-check that the answer differs across verbs and say how you checked.

## Gate

1. `cargo build --release --all-targets` → exit 0, **zero warnings**; `cargo clippy --release
   --all-targets` likewise.
2. `./wat-scripts/perf/grid/check-where-shapes.sh` — 9 pairs, 98 rows, all agreeing. ~35s, no cargo.
   This is STOP-2's proof.
3. `cargo test --release --test lint` — the whole lint target. **Named explicitly because a build-only
   gate is structurally blind to repo lints, and that blind spot has cost two riders today.** If you
   add a test file, `no_loose_string_assert` and `no_inlined_edn` are the two that bite.
4. `cargo test --release --test rete` — the module you are changing.

Do **NOT** run `cargo nextest` — the floor is weighed centrally, once, by me on a quiescent tree
(baseline **4270 passed / 0 failed** at `a787cd25`). Run everything in the FOREGROUND and block on it;
your turn ends when the numbers are in your hands, not when a command is launched.

Do not commit, push, stash, or revert.

## Report

The diff per file; all four gate results; **the enumeration** (verbs, row counts, refused rows) as the
headline; your genuinely-partial vs total-but-unclassified split with the uncertain ones flagged; every
verb you classified `total: true` and the reason; and anything you judged rather than transcribed.
