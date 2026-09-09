# SCORE — STONE A-2 RELAND-5: the runtime must agree

## Census (full detail: `TABLE-STONE-A2-RELAND-5-aggregate-only-runtime-sites.md`)

`grep -n "Value::Aggregate(a)" src/runtime.rs` on the pre-fix tree: **13 sites**, matching the brief
exactly. Classified:

- **4 fix locations needed widening** (8 of the 13 lines — each a two-arm match whose second arm is
  counted with the first): the keyword-as-accessor fall-through (~3660-3693), the `{:keys}`
  `LetBinding::StructDestructure` (~4067), the `{var :field}` `LetBinding::HashDestructure`
  (~4162), and the `{var :field}` match-arm sub-pattern in `try_match_pattern` (~9037). The first two
  are the brief's two named/measured targets; the latter two are the SAME shape (literally reuse
  `keyword_accessor_record`/`keyword_accessor_struct`) and the checker never restricted them in the
  first place (fresh type var per binding, unconditionally), so widening them creates no new
  checker-vs-runtime predicate — just closes the same gap one form over before a future rider tripped
  on it.
- **1 site flagged, NOT fixed**: line 3499 (surface-method dispatch deriving `concrete_type_fqdn`).
  Real gap (`Value::Enum` falls to `other_val.type_name()`, which returns the generic
  `"wat::core::Enum"` rather than the declared FQDN), but it is a type-identity/dispatch defect, not
  a field-reading one — STOP-4 territory. Reported to the orchestrator; not folded into this stone.
- **4 sites classified out of scope, no change**: a doc comment (9082); `conforms_check`'s two
  nature/nominal-identity checks (9721 Record-umbrella test, 9741 unregistered-name fallback — a
  registered variant never reaches this arm); the fixed-shape `Reply::Failed` internal decode
  (14342); a test assertion (14618).

Count of sites needing change: **4** — under the ~6 STOP-3 threshold. No STOP-3.

## What was fixed

1. **`src/runtime.rs`** — new helper `keyword_accessor_enum(bare_name, e: &EnumValue, span)`,
   reading `e.names`/`e.fields` directly (no `TypeEnv` lookup — the brief's point 5: "nothing needs
   to be looked up"). Wired into all four sites via a `Value::Enum(e) => ...` arm alongside the
   existing `Value::Aggregate` arms.
2. **`src/runtime.rs`** — `LetBinding::StructDestructure` (`{:keys}`) restructured: `Value::Aggregate`
   still resolves declared names via the `TypeEnv` (unchanged); `Value::Enum` reads `e.names`/
   `e.fields` straight off the value. Both funnel into one shared name→index→value loop, so the
   lookup/error logic is written once, not duplicated per branch.
3. **`src/check.rs`** — the keyword-as-accessor fall-through's `acceptable` receiver-shape test
   gained TWO arms (not one — see "what nearly slipped through" below): a bare `TypeExpr::Path`
   naming a singleton `TypeDef::Enum`, and a `TypeExpr::Parametric` whose head (via
   `parametric_head_fqdn`) names one. Both test the exact same predicate the `{:keys}` checker path
   already established in RELAND-1 (`Aggregate | singleton Tagged Enum`) — no new predicate invented.
   `MatchArm::HashDestructure`'s checker arm needed no change: it never validated field existence
   against any receiver type (Aggregate, HashMap, or otherwise), so it already admitted a variant.

## What nearly slipped through

The keyword-accessor checker fix's first attempt (Path-only) built and even satisfied `cargo build
--release`, but `./target/release/wat tests/types/probe_arc296_A2_a_variant_is_a_type__field_accessor_on_a_variant_runs.wat`
still failed check-time with the exact `"unknown callee: :inside"` the brief measured — because
`register_variant_types` carries only the type params a variant's OWN fields consume (arc 296 A-2
RELAND-2 mechanism ④), so a **generic** variant's resolved receiver type at the call site is
`TypeExpr::Parametric { head: "usr::Box::Full", args: [i64] }`, not a bare `Path`. This is exactly
the class of defect CLAUDE.md's corollary warns about — a generic form misbehaving via a
string/shape comparison that only covers one spelling. Caught by re-running the RUN row after the
first attempt, not by `--check` alone (a `--check`-only acceptance row would have gone green here
too, on the SAME wrong reason A-2's own nine rows did — the shape it checks is `TypeExpr::Path`
being satisfiable, not "does the actual receiver type resolve").

## STOP triggers

- **STOP-1** (EnumValue shape change): did not fire. `EnumValue.names`/`.fields` used as-is, no
  representation change.
- **STOP-2** (`non_enum_container_stays_invariant` goes green): did not fire.
  `a_non_enum_container_does_not_widen_its_argument` still asserts `code == 1` and passed — the
  fixture is still refused.
- **STOP-3** (census > ~6 sites needing change): did not fire. 4 fix locations.
- **STOP-4** (a site needs a variant arm for a non-field-reading reason): **fired once** — line 3499,
  surface-method dispatch. Reported above and in the table; not fixed.
- **STOP-5** (checker and runtime each encode their own "has named fields" test): did not fire. The
  checker's test is exactly the RELAND-1 predicate (`Aggregate | singleton Tagged Enum`), re-applied
  (not reinvented) at the accessor's `acceptable` match; the runtime's `keyword_accessor_enum` is a
  direct read of value-carried data, with no predicate of its own to duplicate against the checker's
  (it doesn't decide admissibility — the checker already did, at compile time; the runtime just looks
  up the field or raises `UnknownField`).

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'
```
15 tests run: **15 passed**, 0 skipped — including both RUN rows:
- `the_builders_program_runs_and_prints_42` — un-ignored, runs, prints `42`, exit 0.
- `a_field_accessor_works_on_a_variant` — un-ignored, runs, prints `7`, exit 0.

Both `#[ignore]` attributes removed from `tests/types/probe_arc296_a2_a_variant_is_a_type.rs`.

Named unmoved stones, re-run to confirm no regression:

```
p1_annotation        10 passed  (probe_arc296_p1_annotation_names_a_type)
p1b_a_parametric       4 passed  (probe_arc296_p1b_a_parametric_head_is_a_named_type)
p2prereq               4 passed  (probe_arc296_p2prereq_is_type_asks_the_same_union)
p3_one_question        5 passed  (probe_arc296_p3_one_question_one_answer)
a1_one_rule            4 passed  (probe_arc296_a1_one_rule_for_assignability)
```

Additional targeted `-E` filters run as belt-and-suspenders, since this stone's fix touched shared
match blocks (`HashDestructure`, match-arm hash sub-pattern, the accessor fall-through) beyond the
two named targets:

```
struct_destructure                 11 passed
probe_arc278_journal_surface         2 passed
hash_destructure                    12 passed
keyword_accessor                     8 passed
accessor                            59 passed
arc234                              74 passed
arc296                             131 passed
keys_destructure / match_arm        18 passed
```

All green, zero regressions.

## Deltas from the brief, honestly stated

- The census found **2 more field-reading sites than the brief's two named targets**
  (`LetBinding::HashDestructure` and the match-arm hash sub-pattern) and fixed them too, on the
  grounds that they are the identical shape/helpers and the checker never restricted them either —
  leaving them unfixed would have reproduced this stone's own defect one form over. This is a
  judgment call beyond the letter of "your targets are the two measured gaps"; flagging it rather
  than silently expanding scope.
- The checker-side accessor fix required **two match arms, not one** (`Path` and `Parametric`),
  discovered only by running the RUN row after the first (incomplete) attempt passed `cargo build
  --release` — recorded above under "what nearly slipped through" so the next rider doesn't
  re-litigate it.
- One real gap (line 3499, enum method-dispatch FQDN) was found, NOT fixed, and is surfaced here per
  STOP-4 for the orchestrator to shape as its own stone if wanted.

## Not run (per tier)

`scripts/floor.sh`, an unfiltered `cargo nextest run`, and `cargo clippy` were not run, per the
brief's tier instruction. Only the fixtures and the `-E` filters above (all release-mode) were run.
