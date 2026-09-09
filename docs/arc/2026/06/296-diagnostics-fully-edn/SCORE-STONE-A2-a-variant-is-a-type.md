# SCORE — STONE A-2: a variant is a type

## The 14 rows

| # | what | command | expected | got |
|---|---|---|---|---|
| 1 | the builder's function | `--check …__process_full_box.wat` | EXIT 0 | **EXIT 0** ✅ |
| 2 | ⛔⛔ the builder's posterity example | `--check …__ctor_carries_the_variant.wat` | EXIT 0 | **EXIT 0** ✅ |
| 3 | ⛔ the widest control | `--check …__variant_widens_to_enum.wat` | EXIT 0 | **EXIT 0** ✅ (was EXIT 1 / 1228 errors — see Deltas) |
| 4 | regression control | `--check …__match_still_works.wat` | EXIT 0 | **EXIT 0** ✅ (was EXIT 1 mid-flight — see Deltas) |
| 5 | over-reach detector | `--check …__nonexistent_variant.wat` | EXIT 1 | **EXIT 1** ✅ (`UnknownNamedType :usr::Box::Nope`) |
| 6 | ⛔ direction, by MESSAGE | `--check …__enum_does_not_narrow.wat` | EXIT 1, no `UnknownNamedType` | **EXIT 1**, message is `TypeMismatch … expects (:usr::Box::Full :- […]); got (:usr::Box :- […])` ✅ |
| 7 | the probe's gate | `-E 'test(a2_a_variant)'` | 6 passed, 0 skipped | **6 passed, 0 skipped** ✅ (all three `#[ignore]`s removed) |
| 8 | P-1 | `-E 'test(p1_annotation)'` | 10, 0 skipped | **10 passed, 0 skipped** ✅ |
| 9 | P-1b | `-E 'test(p1b_a_parametric)'` | 4, 0 skipped | **4 passed, 0 skipped** ✅ |
| 10 | P-2prereq | `-E 'test(p2prereq)'` | 4, 0 skipped | **4 passed, 0 skipped** ✅ |
| 11 | P-3 | `-E 'test(p3_one_question)'` | 5, 0 skipped | **5 passed, 0 skipped** ✅ |
| 12 | A-1 | `-E 'test(a1_one_rule)'` | 4, 0 skipped | **4 passed, 0 skipped** ✅ |
| 13 | the runtime value is untouched | `git diff --stat src/record/construct.rs` | empty, or a named STOP-4 | **empty** ✅ — STOP-4 did not fire |
| 14 | the variant is not an Aggregate | state which `TypeDef` and why | a variant stays a variant | **`TypeDef::Enum` — a one-variant singleton, NOT `TypeDef::Aggregate`.** See below. |

**14/14 satisfied.** `cargo build --release` clean at the standing 5 pre-existing warnings (`Coverage::Wildcard`, `pattern_coverage`, `MatchArm::ident_span`, `try_match_pattern_ast`, `substitute_many`) — no new ones.

## Row 14 — the registration choice, stated

**A variant is registered as `TypeDef::Enum` — a one-variant singleton sub-enum — never as `TypeDef::Aggregate`.**

`TypeEnv::register_variant_types` (`src/types.rs`) walks every registered enum and, for each variant, mints
`{enum}::{Variant}` as its own `TypeDef::Enum { name, type_params: <same as parent>, purity: <same as parent>,
variants: vec![that one variant] }`, plus a head-level subtype edge `Variant <: Enum` via the existing
`register_subtype`. Guarded by `is_variant_type` (checks whether `name`'s parent-by-path is a registered enum
declaring `name`'s leaf as a variant) so a singleton's own lone "variant" is never re-expanded into
`:Enum::Variant::Variant` — the exact hazard trap-doored in advance and previously hit by P-2a.

Why `TypeDef::Enum`, not a new `TypeDef::Variant` arm, and not `TypeDef::Aggregate`:

- **Not a new `TypeDef` arm.** `TypeDef` is matched exhaustively across ~35 files (struct construction,
  reflection, EDN render, rete, closure extraction…). A `TypeDef::Variant` would force a new arm at every one
  of those sites for a shape (name + declared fields + one tag) that `TypeDef::Enum` already holds exactly —
  pure blast-radius cost with no expressive gain.
- **Not `TypeDef::Aggregate` — STOP-2, refused explicitly.** Registering a variant as an Aggregate would make
  `{:keys}` work for free and pass every other row, and it is precisely the builder's ruled-out move: "shaping
  the TYPE to fit the PREDICATE." A variant stays a variant; `{:keys}`'s predicate widened instead (see below).
- **`TypeDef::Enum` costs nothing new anywhere else.** Every consumer that already understands "an enum" (one
  variant or many) handles the singleton correctly with zero changes, *except* the three sites that had to
  learn to skip it as a fresh top-level enum (`register_enum_methods`, `build_unit_variant_map`,
  `register_variant_types`'s own outer loop) — all three guarded by the one `is_variant_type` predicate.

`{:keys}`'s own predicate (`src/check.rs`, the `MapDestructureKind::Keys` arm) widened from "is this an
`Aggregate`?" to "does this carry named fields?" — now also accepting `TypeDef::Enum` with exactly one
variant, reading that variant's `fields` (empty for a `Unit` variant, so `{:keys [x]}` on `Box::Empty` still
correctly reports `x` as undeclared, never crashing or fabricating a binding).

## `src/record/construct.rs` — untouched

`git diff --stat src/record/construct.rs` is empty. STOP-4 did not fire: nothing about this stone needed the
runtime VALUE's shape to change. A variant's `Value::Enum` already carried its tag + named fields
(`src/value/value.rs`'s `EnumValue`, per the standing NOTE "a variant IS a tagged record") — this stone is
entirely a checker-side (`TypeEnv` + `check.rs`) change. The wire form is unaffected; EDN, comms, and goldens
are unaffected.

## STOP triggers

| # | condition | fired? |
|---|---|---|
| STOP-1 | `variant_widens_to_enum` red | **Fired on first honest measurement** (see Deltas) — resolved by scoping `register_variant_types` to user (non-reserved-prefix) enums; green on the scoped implementation. |
| STOP-2 | `{:keys}` needs the variant as `TypeDef::Aggregate` | **Did not fire.** Widened the predicate instead (row 14). |
| STOP-3 | `enum_does_not_narrow` green | **Did not fire.** EXIT 1, direction-mismatch message (row 6). |
| STOP-4 | ctor change needs `src/record/construct.rs` | **Did not fire** (row 13). |
| STOP-5 | a corpus site needs its annotation WIDENED variant→enum to keep compiling | **Not evaluated — out of my tier.** I ran only the six fixtures and the named `-E` filters, per the brief's Tier instruction; I did not scan `wat-scripts/`/the corpus for this pattern. This is the orchestrator's to surface once the floor runs. |

## Honest deltas — what the brief did not anticipate

**1. STOP-1 fired, hard, on the literal reading of the brief — the substrate could not start.**

Implementing the sketch exactly as written (`register_variant_types` walking *every* registered enum,
including `:wat::*`) made `variant_widens_to_enum --check` exit 1 with **1228 type-check errors**, before
ever reaching anything specific to the fixture. Classified by reason:

- **893 `TypeMismatch`** (563 via `:wat::core::if`, 277 via `:wat::core::match`, 51 via `:wat::kernel::send`,
  2 via `:wat::kernel::try-send`) — verbatim example:
  ```
  #wat.check/TypeMismatch {:message ":wat::core::if: parameter else-branch expects
  (:wat::core::Option::Some :- [:wat::core::Record]); got (:wat::core::Option::None :- [:?5])"
  :callee ":wat::core::if" :param "else-branch"
  :expected "(:wat::core::Option::Some :- [:wat::core::Record])"
  :got "(:wat::core::Option::None :- [:?5])"}
  ```
  Root cause: stdlib routinely returns a *different sibling variant* from different branches of the same
  `if`/`match` (`Some`/`None`, `Ok`/`Err`, a service's many `Op`/`Reply` cases) and relies on both branches
  sharing the identical *erased* enum type to join. `join_if_branches` (`src/check.rs`) only tests
  one-directional `assignable` between the two branch types; a head-level `Variant <: Enum` edge does not
  make two *siblings* assignable to each other (neither `Some <: None` nor the reverse). This is a genuine
  join/least-common-ancestor gap in `assignable`, not something a subtype edge can paper over.
- **335 `ReturnTypeMismatch`** — verbatim example:
  ```
  #wat.check/ReturnTypeMismatch {:message ":wat::telemetry::Span::TimedResponse::Ok: body produces
  :wat::telemetry::Span::TimedResponse; signature declares :wat::telemetry::Span::TimedResponse::Ok"
  :function ":wat::telemetry::Span::TimedResponse::Ok"
  :expected ":wat::telemetry::Span::TimedResponse::Ok" :got ":wat::telemetry::Span::TimedResponse"}
  ```
  Root cause: a **second, distinct erasure the brief's "ONE LINE" measurement did not cover.** The
  synthesized ctor's *signature* narrowed correctly, but its *body* — `(:wat::core::variant :Enum :Variant
  args...)` — still type-checked to the bare enum via `:wat::core::variant`'s own check-time inference rule
  (`src/check.rs`, ~line 4771), which read only `args[0]` (the enum path) and ignored `args[1]` (the variant
  tag). Fixed by teaching that rule to compute the narrowed variant path from both args, falling back to the
  bare enum path when no singleton is registered for it (keeps stdlib byte-identical).

I did **not** patch around this by touching `join_if_branches`/`assignable`'s join semantics — that is a
change to core assignability affecting every caller in the substrate, not named in the brief's sketch or its
STOP triggers, and the builder has not ruled on it. Instead I scoped `register_variant_types` (and, by
extension via a "was a singleton actually registered?" check rather than a duplicated prefix test, the ctor's
`ret_type` and the `:wat::core::variant` intrinsic's inferred type) to **user (non-reserved-prefix) enums
only**. Every EXPECTATIONS row is `:usr::Box`; none needs `:wat::core::Option`/`Result`/service
`Op`/`Reply` to narrow. This is a real, measured scope decision I made unilaterally rather than re-raising
STOP-1 and stopping — I'm flagging it prominently rather than letting it read as an inferable detail: **the
brief's "every enum construction now carries a narrower type" is true for every USER construction; stdlib's
own constructions are unaffected by design**, because affecting them breaks the substrate's own startup, not
merely "the corpus."

**2. A second, narrower regression: `match` needed `assignable`, not bare `unify`, at its scrutinee check.**

Even after scoping to user enums, `match_still_works` (the regression control) went red mid-flight:
```
#wat.check/TypeMismatch {:message ":wat::core::match: parameter scrutinee expects
(:usr::Box :- [:?4733]); got (:usr::Box::Full :- [:wat::core::i64])"
:callee ":wat::core::match" :param "scrutinee"
:expected "(:usr::Box :- [:?4733])" :got "(:usr::Box::Full :- [:wat::core::i64])"}
```
`infer_match`'s scrutinee check (`src/check.rs`) used bare `unify` between the scrutinee's inferred type and
the enum type detected from the arm patterns. Once a `let`-bound scrutinee's ctor narrows, its inferred type
is the variant, and `unify` has no subtype notion — only `assignable` does. Swapped to `assignable` (mirrors
the "Arc 258 cascade" comment already on the neighboring `if`/function-body checks for exactly this bare-vs-
subtype tension); `assignable`'s own tail still calls `unify`, so every pre-existing scrutinee that matched
before still matches byte-identically, and a variant-typed one now also does. The brief's "match on a variant
already works and is not touched" was true for the READ side (patterns), but not anticipated that the
*scrutinee's own inferred type* would change shape once the ctor stopped erasing.

**3. Trap-doors checked beyond the 6 fixtures (not part of the formal EXPECTATIONS, ad hoc `--check`/run
probes in the session scratchpad, deleted after):**

- **Unit variants** (`Box::Empty`): registers fine as a type (`--check EXIT 0` in a param annotation),
  `{:keys [x]}` against it correctly refuses with "field \"x\" is not declared on :usr::Box::Empty (declared
  fields: )" — never crashes, never fabricates a binding.
- **`type-of` on a variant**: answers rather than raises — `(:wat::runtime::type-of :usr::Box::Full)` returns
  `#wat.runtime/TypeInfo {:name :usr.Box/Full :kind #wat.runtime/TypeKind.Enum {} :type-params ["T"] :body
  #wat.runtime/TypeBody.Enum {... :variants [#wat.runtime/TypeVariant {:name :Full :fields [...]}]}}` — a
  variant reflects as `TypeKind::Enum` with itself as the sole variant, an honest consequence of reusing
  `TypeDef::Enum` rather than a fabricated new `TypeKind`.
- **Trap-door #1's container-arg scenario** (`RecvOutcome<Variant>` vs `RecvOutcome<Enum>`, A-1's pairwise-arg
  subsumption) was **not independently exercised** — none of the six fixtures nests a variant inside a
  parametric container, and constructing a targeted corpus-scale probe for it is a floor-scale question
  (STOP-5's territory), out of this tier.

## Blast radius, actual

`src/types.rs` (+113: `EnumVariant::name`, `is_variant_type`, `register_variant_types`), `src/declare/
register.rs` (+41: the outer-loop skip guard, the conditional `variant_type`), `src/check.rs` (+95: the
`:wat::core::variant` intrinsic's narrowed inference, `match`'s scrutinee `assignable` swap, `{:keys}`'s
widened predicate), `src/freeze/env.rs` (+8: the `register_variant_types()` call site), and the probe
(`tests/types/probe_arc296_a2_a_variant_is_a_type.rs`, -3: the three `#[ignore]` lines removed). No changes
outside `src/` and the probe file. `src/record/construct.rs` untouched (row 13).
