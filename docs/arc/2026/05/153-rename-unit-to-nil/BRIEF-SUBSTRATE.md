# Arc 153 — Substrate BRIEF (slice 1a)

**Drafted 2026-05-06.** Slice 1a substrate work for arc 153.

User direction (verbatim):
> *"name swap first, then close out the do forms. do forms being
> resolved depends on nil being valid and expressed correctly
> throughout the code base"*

> *"we ride to compaction - we are the best at what we do - we are
> doing as fast as we can - i need a lisp on rust to satisfy what
> we're building towards"*

## Workspace state pre-spawn
- HEAD: `4029173` (arc 153 DESIGN just committed + pushed)
- Working tree clean
- Pre-baseline: `cargo test --release --workspace` = 1978 passed / 0 failed

## Goal

Two coordinated substrate changes:

1. **Type-position rename** — retire `:wat::core::unit`; mint
   `:wat::core::nil` as the canonical FQDN for the singleton type.
   Per substrate-as-teacher Pattern 3 (symbol migration with
   walker), every existing `:wat::core::unit` annotation site
   fires a `BareLegacyUnitName` migration error; sweep 1b migrates
   consumers.

2. **Value-position recognition** — `:wat::core::nil` keyword at
   value position parses as the nil-value literal (types as
   `:wat::core::nil`; evaluates to the nil singleton). This is
   ADDITIVE (doesn't break consumers); enables sweep 1b's
   value-position transform `()` → `:wat::core::nil`.

## Substrate edits

### `src/types.rs`

Add `CheckError::BareLegacyUnitName` variant:

```rust
/// Arc 153 — `:wat::core::unit` retired in favor of
/// `:wat::core::nil`. Same role (singleton, "no meaningful
/// return value"); rename ships the marker effect of a Lisp's
/// `nil` while preserving the type-system split with
/// `:wat::core::None` (Option's absence).
///
/// Detected via TypeExpr walker (Pattern 3 per arc 109 slice 1d's
/// `BareLegacyUnitType` precedent).
BareLegacyUnitName {
    span: Span,
},
```

Display: `"{span}: ':wat::core::unit' is retired (arc 153);
canonical FQDN is ':wat::core::nil'. Same role, same type
properties, new name."`

### `src/check.rs`

1. **Walker** — extend `validate_bare_legacy_unit_type` (or add a
   new `validate_bare_legacy_unit_name`) to detect
   `TypeExpr::Path(":wat::core::unit")` and emit
   `BareLegacyUnitName`. Mirror arc 109 slice 1d's
   `BareLegacyUnitType` shape (which detected `()` shape).

2. **Mint `:wat::core::nil` type** — add `:wat::core::nil` as a
   recognized canonical FQDN; the type registry should resolve
   `:wat::core::nil` to the singleton type previously named unit.
   Reuse the underlying `TypeDef` representation; only the name
   changes.

3. **Value-position recognition** — in `infer` for
   `WatAST::Keyword(s, _)` where `s == ":wat::core::nil"`, return
   the nil type (instead of `:wat::core::keyword`). This
   special-cases the FQDN keyword string at value position.

### `src/runtime.rs`

In `eval` for `WatAST::Keyword(s, _)` where `s ==
":wat::core::nil"`, return `Value::Unit` (or whatever the nil
singleton's runtime representation is). Keep all other keyword
paths unchanged.

### NEW `tests/wat_arc153_nil_rename.rs`

6-10 unit tests:

1. **Type-position retired:** `(:wat::core::define (:probe ->
   :wat::core::unit) ...)` fires `BareLegacyUnitName` migration
   error
2. **Type-position canonical:** `(:wat::core::define (:probe ->
   :wat::core::nil) ...)` works
3. **Value-position works:** `(:wat::core::define (:probe ->
   :wat::core::nil) :wat::core::nil)` type-checks + evaluates
4. **Type-mismatch:** declaring `-> :wat::core::i64` but body is
   `:wat::core::nil` fires TypeMismatch
5. **Mixed `()` and `:wat::core::nil`:** body returns `()` while
   sig declares `-> :wat::core::nil` — type-checks (both produce
   the singleton)
6. **Reverse mixed:** body returns `:wat::core::nil` while sig
   declares `-> :wat::core::unit` — fires
   `BareLegacyUnitName` on the sig (the `unit` retired); body
   side is fine
7. **Reflection round-trip:** lookup-form / signature-of returns
   `:wat::core::nil` (post-rename canonical)
8. **Eval observable:** the evaluated value of `:wat::core::nil`
   in a do form is the nil singleton (verified via assert-eq
   against `()`)
9. **HashMap key context (regression check):** a HashMap with
   keyword keys still treats other keywords (e.g.,
   `:user::foo`) normally — the special case is narrow to
   `:wat::core::nil`
10. **Macro round-trip (if reachable):** the AST rewrites preserve
    `:wat::core::nil` keyword form

Use the existing test harness pattern (`check_errors`, `eval`,
etc.) from `tests/wat_arc145_typed_let.rs`'s shape (note: that
file was reverted; reference the pattern from
`tests/wat_arc136_do_form.rs` instead).

## Constraints

- **Substrate-only edits.** EXACTLY 4 files: `src/types.rs`,
  `src/check.rs`, `src/runtime.rs`, NEW
  `tests/wat_arc153_nil_rename.rs`. NO consumer wat edits. NO
  other crate.
- **DO NOT COMMIT.** Working tree stays modified; orchestrator
  commits sweep 1a + sweep 1b atomically when workspace =
  0-failed (per recovery doc § 7
  atomic-commit-across-coordinated-sweeps).
- **The workspace WILL break post-substrate-change** — every
  existing `:wat::core::unit` annotation in stdlib + tests fires
  the migration error. THIS IS EXPECTED. Sweep 1b runs
  immediately after.
- **STOP at first unexpected red.** Distinguish:
  - **Expected red:** `BareLegacyUnitName` on existing
    `:wat::core::unit` sites
  - **Unexpected red:** anything else (substrate panic, parse
    error inside check.rs/runtime.rs, runtime crash, TypeMismatch
    not tracing to nil/unit interaction)
- No grinding.

## Pre-flight crawl (mandatory)

1. **`docs/arc/2026/05/153-rename-unit-to-nil/DESIGN.md`** — full
   read; especially "Substrate work" + "Slice plan"
2. **`docs/arc/2026/04/109-kill-std/INVENTORY.md`** § A — locate
   `BareLegacyUnitType` walker precedent (arc 109 slice 1d)
3. **`src/check.rs::BareLegacyUnitType`** + its
   `validate_bare_legacy_unit_type` walker — your canonical
   pattern reference; mirror it for the new variant
4. **`src/check.rs` keyword inference path** — find where Keyword
   ASTs get their type today (likely `infer` arm matching
   `WatAST::Keyword`); your value-position special-case lives
   there
5. **`src/runtime.rs` keyword eval path** — find where Keyword
   ASTs get evaluated; your value-position handler lives there
6. **`tests/wat_arc136_do_form.rs`** — current canonical test
   harness pattern (use this as the shape template)

## Pre-flight verification (test BEFORE editing)

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0.

## Verification (after edits)

After your substrate edits:

```bash
cargo test --release --test wat_arc153_nil_rename 2>&1 | tail -10
```

Expect: all 6-10 new tests pass.

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail -5
```

Expect: many CONSUMER tests fire `BareLegacyUnitName` migration
errors on existing `:wat::core::unit` sites. These are expected
per atomic-commit pattern; sweep 1b will resolve.

## Out of scope

- Sweep 1b (consumer migration) — separate brief
- Slice 2 closure paperwork — out of scope here
- Lab consumers (`holon-lab-trading/`) — separate workspace

## Reporting (~250 words)

1. **Pre-flight crawl confirmation:** DESIGN, BareLegacyUnitType
   walker precedent, keyword infer/eval paths,
   tests/wat_arc136_do_form.rs all read.

2. **Edit summary:**
   - `BareLegacyUnitName` variant added to `CheckError`
   - Walker arm detects `:wat::core::unit` type-position; emits
     migration error
   - `:wat::core::nil` canonical type minted
   - Keyword infer/eval special-case for `:wat::core::nil` at
     value position
   - 6-10 new tests in `tests/wat_arc153_nil_rename.rs`

3. **LOC delta:** before/after across the 4 files.

4. **Verification:**
   - `cargo test --test wat_arc153_nil_rename` — all pass
   - `cargo test --workspace` — failure profile
     (many `BareLegacyUnitName` consumer errors expected)

5. **Path:** Mode A clean (substrate ships; consumer failures match
   expected `BareLegacyUnitName` shape) / Mode B
   substrate-internal-bug / Mode C unexpected-failure-shape.

6. **Honest deltas:** any subtleties around the keyword
   value-position special-case (e.g., HashMap-key contexts);
   any edge case in the walker (parametric types containing
   `:wat::core::unit`); any other surface area beyond brief
   enumeration.

DO NOT write a SCORE doc — orchestrator's work after sweep 1b
ships and atomic commit lands.

## Time-box

60 minutes wall-clock (predicted upper-bound 30-45 min; 2× cap).

## Why this matters

User direction 2026-05-06: "name swap first." Slice 1a substrate
ships the rename; slice 1b sweeps consumers; arc 136 slice 2 (do
form closure) waits on arc 153 because the do form's return
positions become `:wat::core::nil` after the sweep.

The rename ships the marker effect of `nil` while preserving
wat-rs's existing nil/None split (Option<T>::None for absence;
nil for "no meaningful return value"). The triplet `nil / Some /
None` reads cleanly at every consumer site.
