# BRIEF — STONE A-2 RELAND-3: the class, not the sites

## Start here

**`git show 60813552a` then `git show <the RELAND-2 WIP>`** — restore the accumulated work. The join
is sound; three mechanisms are closed. This stone closes what the floor named next, and its FIRST
act is a census, because the last two briefs named SITES where the finding was a CLASS.

## ① THE CENSUS COMES FIRST — this is the stone's real deliverable

RELAND-2 fixed `infer_option_expect` and `infer_result_expect` because the floor named them. The
next floor named the comparison family. `src/check.rs` has **73 `infer_*` functions**, each a
hand-written parameter check. Some use `assignable`; some use bare `unify`; a user `defn` always
uses `assignable`.

**Census every one of them.** For each `infer_*` that compares an argument's type against a declared
or expected parameter type, record: the function, whether it uses `unify` or `assignable`, and
whether a user-supplied enum value can reach it. Write the table to
`docs/.../TABLE-STONE-A2-RELAND-3-intrinsic-parameter-checks.md`.

⚠ **Report the count before fixing anything.** If it is large, that is a finding and the orchestrator
re-plans — do not silently sweep 73 functions.

## ② The comparison family — TWO sub-defects, both at `src/check.rs:~13822`

```
":wat::core::<: parameter #2 expects (:wat::core::Option :- [:?N]); got (:wat::core::Option::Some :- [i64])"
":wat::core::>=: parameter #1 expects an orderable type (…, (Option :- [T]), (Result :- [T E]));
                             got (:wat::core::Result::Err :- [:?N :wat::core::String])"
```

- the operand comparison calls `unify(&a_resolved, &b_resolved, …)` directly — route it as a user
  defn's parameter would be routed;
- `is_type_orderable` lists `(Option :- [T])` and `(Result :- [T E])` but not a **variant** of one.
  A variant of an orderable enum is orderable — it is the same values.

## ③ A golden that pins the ERASED spelling — recapture, do not normalise

`tests/services/probe_arc278_journal_surface_swap.wat.bad` asserts:

```
expected == ":wat::telemetry::Journal::WriteMetricsResponse" && got == ":wat::query::Store::PutResponse"
```

The `.wat.bad` program still fails, correctly. The message now names the **variant** —
`::PutResponse::Success` — which is strictly more informative. **RECAPTURE the assertion, KEEP IT
PINNING.** The seam's standing ruling: *"A golden pinning a stdlib line: RECAPTURE, KEEP PINNING.
Never extend the normaliser."*

## ⛔ STILL FENCED — mechanism ② (nested widening), 7 failures

```
expects  (Option       :- [(Result      :- [i64 String])])
got      (Option::Some :- [(Result::Err :- [:?N String])])
```

Head **and** argument. `assignable`'s per-argument check is invariant `unify`, one level down, by
design. **The builder has not ruled whether an argument position may widen.** Do not touch it, do
not relax invariance, do not route around it. If your work makes these pass as a side effect, say
so explicitly and show why — that is a finding, not a silent win.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'   11 passed, 0 skipped
```

Earlier stones unmoved: `p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 ·
`p3_one_question` 5 · `a1_one_rule` 4. Plus the census table on disk.

## STOP triggers — each is a REJECTION

**STOP-1.** If the census exceeds ~15 functions needing change — STOP after the census and report it.
That is a campaign, not a stone, and it is the orchestrator's to shape.

**STOP-2.** If either join control goes red — STOP with the verbatim block.

**STOP-3.** If mechanism ② starts passing — STOP and explain the mechanism. It is fenced pending a
ruling; passing it accidentally is as much a finding as failing it.

**STOP-4.** No scoping by namespace or prefix. Two relands died on that.

**STOP-5.** If fixing `is_type_orderable` requires enumerating variants by name rather than asking
whether the enclosing enum is orderable — STOP and report. A hand-list is what this campaign deletes.

## Tier

You edit and report. Run the fixtures and the targeted `-E` filters. **You do NOT run
`scripts/floor.sh`, an unfiltered `nextest`, or clippy.** Classify anything remaining BY REASON with
one verbatim block per cluster.
