# Arc 198 Slice 2 Stone 2 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-2-PROC-MACRO-ATTRIBUTE.md`

## Independent prediction

**Runtime band:** 90 minutes sonnet.

Reasoning:
- Proc-macro attribute parser in wat-macros: ~50-80 LOC (existing `#[wat_dispatch]` is the template)
- Codegen emitting `inventory::submit!` block: ~30-50 LOC
- 3 new tests in dedicated file: ~120-180 LOC
- Cargo.toml updates if any (likely none — wat-macros already has syn/quote)

**Time-box:** 120 min hard stop.

## SCORE methodology

5 rows YES/NO per BRIEF:

- **Row A** (attribute defined): `grep -nA 5 "restricted_to" crates/wat-macros/src/lib.rs` shows registration + parser
- **Row B** (codegen): grep `inventory::submit\|RestrictionEntry` in `crates/wat-macros/src/` shows codegen path
- **Row C** (variadic): tests with 1 and 2+ prefixes both pass
- **Row D** (3 tests pass): `cargo test --release -p wat --test wat_arc198_slice2_stone_2_attribute` → all green
- **Row E** (workspace baseline): cargo test summed failed ≤ baseline + flake variance

## Honest deltas to watch for

- **Sub-decision (a) positional vs (b) named.** Default (a) per Rust attribute idioms. If (b) reads dramatically cleaner (e.g., the multi-prefix array syntax becomes awkward positionally), surface in SCORE.

- **`RestrictionEntry` field type compatibility.** Stone 1 defined fields as `&'static str` (literal-friendly). String literals in attributes ARE `&'static str` after stringification, but verify in codegen — may need explicit lifetime annotations or stringification handling.

- **Path resolution for `wat::RestrictionEntry`.** Generated code lives in the CONSUMER crate (wat). The codegen needs to reference `RestrictionEntry` via a path that resolves from the consumer's namespace — typically `::wat::RestrictionEntry` (absolute) or `wat::RestrictionEntry`. Sonnet to verify.

- **Inventory submit at attribute application site.** The `inventory::submit!` macro produces a `static` item; it must land at module scope, not inside the fn body. Codegen should emit it adjacent to the fn (before or after) in the same module scope.

- **Hygienic identifier generation.** If the codegen needs a unique static name (e.g., `__RESTRICTION_FOR_<fn_ident>`), use the fn ident to generate it. Existing `#[wat_dispatch]` is the precedent for hygienic naming.

- **Pre-existing test failures.** Per Stone 1 SCORE: 3 stable failures (t6, totally_bogus, startup_error) + 1 lifeline flake within rotation band.

## Workspace baseline (commit `51c69a1`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3-4 failures (3 stable + lifeline flake variance)

Post-Stone-2 target:
- ≥ baseline + 3 passed (3 new tests)
- ≤ baseline failures (purely additive change)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 3 | TBD |
| Sub-decision chosen | (a) positional OR (b) named | TBD |
| Substrate-discovery surprises | 0-3 | TBD |
| Mode | Additive (proc-macro attribute + 3 verification tests) | TBD |
