# BRIEF — STONE A-2 RELAND-2: the four mechanisms the join does not reach

## Start here

**`git show 60813552a`** — RELAND-1's preserved work. The join is sound and stays: `join_types`
shared by `join_if_branches`, `combine_match_arm` and `relate_value_to_slot`; structural; consults
`TypeEnv::enclosing_enum`, never `subtype_edges`; no scope cut; `construct.rs` untouched. **Restore
it and build on it.** All 15 of its acceptance rows were green. This stone closes the four mechanisms
the central floor found underneath them — 40 failures, classified by reason.

## The four, in the order I would take them

**① Two parameter-checking paths — 17 of 40.** The largest, and not variance:

```
user defn      [o <- (:wat::core::Option :- [:T])]  ←  (Option::Some {:value 42})   check=0
Option/expect  (an intrinsic)                        ←  the same value              check=1
  ":wat::core::Option/expect: parameter opt expects (:wat::core::Option :- [:?1633]);
                                                got (:wat::core::Option::Some :- [:wat::core::i64])"
```

A user `defn`'s parameter check routes through `assignable`; an intrinsic's scheme-based check does
not. Find where a scheme's parameters are compared and route it through the same door. `Result/expect`
(11) and `Option/expect` (6) are the same site.

**② Nested widening — 7 of 40.** ⚠ **This one needs a design answer; do not invent one.**

```
expects  (Option       :- [(Result      :- [i64 String])])
got      (Option::Some :- [(Result::Err :- [:?N String])])
```

Head **and** argument both need widening. `assignable`'s per-argument check is deliberately
invariant. If ① alone does not clear these, **STOP and report** — whether an argument position may
widen is the builder's to rule, and RELAND-1's own SCORE flagged the same tension.

**③ A channel's message var pins to the first payload — 5 of 40.**

```
":wat::kernel::send: parameter payload expects :probe::Msg::Setup; got :probe::Msg::Work"
```

The channel's message-type variable was solved to the FIRST concrete payload's *narrow* type
instead of its enclosing enum. Where a payload's type is used to pin a channel var, pin the
enclosing enum.

**④ A unit variant inherits an unused type param — 3 of 40.**

```
UnconsumedTypeParam: "T" in :probe::counter::Status::Hibernated's param-spec is declared but never used
```

The singleton copies the parent enum's `type_params` onto a variant with no fields to consume them.
Carry only the params the variant's own fields actually use.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'   11 passed, 0 skipped
```

Every fixture in `tests/types/probe_arc296_A2_*` at its stated exit, plus the earlier stones
unmoved: `p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 · `p3_one_question` 5 ·
`a1_one_rule` 4.

⚠ `intrinsic_param_accepts_a_variant` is **green on main and red at `60813552a`** — it is vacuous
without variant types, and its own doc comment says so. Judge it against the restored tree.

## STOP triggers — each is a REJECTION

**STOP-1.** No scoping variant registration by namespace or prefix. RELAND-0 did that; the floor
showed it relocated the failures and cut `Option`/`Result`.

**STOP-2.** If either join control (`sibling_variants_join`, `…_in_match`) goes red — STOP with the
verbatim block. They are green today and breaking them is the 600-failure regression.

**STOP-3.** If mechanism ② is not cleared by ① — **STOP and report**, do not relax argument-position
invariance on your own judgement. That is a language decision the builder has not made.

**STOP-4.** If the join needs to search `subtype_edges` — STOP. 24 of its 34 edges are `extend-type`
protocol satisfaction.

**STOP-5.** If `src/record/construct.rs` must change — STOP. Two relands proved this is checker-side.

⚠ **On name-minting:** `crates/wat-reader/src/identifier.rs` has only DECOMPOSITION accessors —
`leaf`, `path`, `namespace`, `receiver`, `method`, `bare` — and no composition function. Use
`identifier.rs` to PARSE; `format!` matching `register_enum_methods`' existing `constructor_path`
idiom is the only available way to BUILD, and is not a violation. A previous brief of mine forbade
it and was unsatisfiable.

## Tier

You edit and report. Run the fixtures and the targeted `-E` filters. **You do NOT run
`scripts/floor.sh`, an unfiltered `nextest`, or clippy** — the orchestrator runs those centrally,
once. Classify anything remaining BY REASON with one verbatim block per cluster; a count alone is
not a report.
