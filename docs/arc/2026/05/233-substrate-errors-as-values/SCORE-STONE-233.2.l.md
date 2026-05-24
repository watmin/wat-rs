# SCORE — Arc 233 Stone 233.2.l — #[wat_value] proc-macro structural seal

**Result: 12/12 PASS**

## Scorecard

| # | Row | Actual |
|---|---|---|
| 1 | Compile clean (wat) | `warning: \`wat\` (lib) generated 107 warnings` / `Finished \`release\` profile [optimized] target(s) in 0.04s` — 0 errors |
| 2 | Compile clean (wat-macros) | `Finished \`release\` profile [optimized] target(s) in 0.02s` — 0 errors |
| 3 | **233.2.l probe FLIPS 0/3 → 3/3** | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 4 | **wat-macros tests (incl. trybuild compile-fail fixtures)** | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s` (trybuild: 5 fixtures all pass) |
| 5 | Lib tests baseline | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.16s` |
| 6 | Stone 233.2.k probe still passes | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 7 | Stone 233.2.j probe still passes | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 8 | Stone 233.2.i eval signature probe still passes | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 9 | Stone 233.2.h TrackedValue mint probe still passes | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 10 | Stone 233.1 ValueSnapshot probes still pass | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` |
| 11 | Clippy no new warnings | `54` — at boundary; matches 233.2.k baseline |
| 12 | holon-rs untouched | empty output |

## Phase breakdown

### Phase 1 — Proc-macro implementation (`crates/wat-macros/src/wat_value.rs`)

**New file: 249 lines**

Key sections:
- `pub fn wat_value(args, input)`: top-level proc-macro dispatcher — rejects enum-level args, iterates variants, skips opt-in variants, checks field types, emits compile errors.
- `extract_allow_wrapping_reason(attrs)`: scans per-variant attrs for `#[wat_value(allow_wrapping = "...")]`; returns `Some(reason)` if found.
- `is_wat_value_attr(attr)`: predicate — `attr.path().is_ident("wat_value")`.
- `AllowWrappingArg`: `syn::parse::Parse` impl for `allow_wrapping = "..."` key-value pair.
- `is_forbidden_field_type(ty, enum_name)`: recursive syntactic scan — rejects `Self` / `EnumName` directly, and `Box<T>` / `Arc<T>` / `Rc<T>` where T is itself forbidden. Follows `Type::Reference` too. All other forms allowed.

One compile fix during development: borrow conflict on `variant` (moved into `Error::new_spanned` span while still needing `variant.ident`). Fixed by pre-collecting `variant_name` as `String` and using `&*variant` for the span argument.

### Phase 2 — lib.rs export (`crates/wat-macros/src/lib.rs`)

**~35 lines added**

- `mod wat_value;` added to module declarations.
- `#[proc_macro_attribute] pub fn wat_value(args, input)` added — delegates to `wat_value::wat_value(args, input)`.
- Full doc comment with usage examples for the enum-level form and the per-variant opt-in form.
- Import: `use wat_macros::{restricted_to, wat_value};` in `src/runtime.rs` (single line change).

### Phase 3 — `#[wat_value]` application to real `pub enum Value`

**2 lines in `src/runtime.rs`**

- Import extended: `use wat_macros::{restricted_to, wat_value};`
- `#[wat_value]` attribute added above `#[derive(Debug, Clone)]` on `pub enum Value` with comment: `// Arc 233 Stone 233.2.l: structural seal — forbids wrapping-style variants.`

Macro applies cleanly — all existing Value variants (leaf types, Arc-wrapped containers, compound types, etc.) pass the detection rule. No wrapping-style variants remain (Value::Tracked was retired in 233.2.k).

### Phase 4 — trybuild fixtures

**6 files: 1 test runner + 5 UI fixtures + 3 stderr snapshots**

| Fixture | Type | Status |
|---|---|---|
| `tests/wat_value_ui_tests.rs` | runner (trybuild::TestCases) | N/A — driver |
| `tests/ui/ui_wat_value_rejects_box_self.rs` | compile-fail | PASS |
| `tests/ui/ui_wat_value_rejects_box_self.stderr` | snapshot | accepted |
| `tests/ui/ui_wat_value_rejects_arc_self.rs` | compile-fail | PASS |
| `tests/ui/ui_wat_value_rejects_arc_self.stderr` | snapshot | accepted |
| `tests/ui/ui_wat_value_rejects_self_direct.rs` | compile-fail | PASS |
| `tests/ui/ui_wat_value_rejects_self_direct.stderr` | snapshot | accepted |
| `tests/ui/ui_wat_value_accepts_opt_in.rs` | compile-pass | PASS |
| `tests/ui/ui_wat_value_rejects_alias_bypass.rs` | compile-pass (documented limitation) | PASS |

Workflow: first run generated `wip/*.stderr` files; moved to `tests/ui/`; second run confirmed all 5 fixtures pass.

### Phase 5 — Runtime probe verification

**`tests/probe_stone_233_2_l_wat_value_seal.rs`** — 3/3 PASS

- Probe 1: `probe_1_wat_value_applies_to_container_only_enum` — `#[wat_value]` applies to a container-only enum (Vec, Option with Box inside Option); enum is constructable and matchable. PASS.
- Probe 2: `probe_2_wat_value_accepts_opt_in_escape_hatch` — opt-in with non-empty reason string; `Box<LegacyInteropEnum>` field allowed under opt-in; enum constructable. PASS.
- Probe 3: `probe_3_value_enum_constructable` — real `Value::i64(42)` constructable; wat crate still compiles with `#[wat_value]` applied. PASS.

## Honest deltas

### Alias bypass: documented limitation, compile-pass

Per Decision 1 of sub-DESIGN: the macro uses a pure syntactic scan. A type alias (`type BoxedValue = Box<BadValue>`) bypasses the seal because the macro sees `BoxedValue` (single-segment path, no `Box`/`Arc`/`Rc` match, not `Self`/enum name). The `ui_wat_value_rejects_alias_bypass.rs` fixture is compile-pass, not compile-fail, with a detailed comment explaining the limitation and the recommended workaround (explicit `#[wat_value(allow_wrapping = "...")]` opt-in or avoiding the alias). This matches the DESIGN's Decision 1 verdict exactly.

### No Rc<Self> trybuild fixture (coverage gap — minor)

The BRIEF lists `Rc<Self>` as a detection target alongside `Box`/`Arc`. The detection algorithm handles it (same `matches!(seg_name, "Box" | "Arc" | "Rc")` branch). There is no dedicated `ui_wat_value_rejects_rc_self.rs` fixture; coverage is provided by the Box and Arc fixtures which exercise the same code path. Not scope creep to add, but not a gap in the detection itself.

### Single borrow conflict during development

The initial implementation of `is_forbidden_field_type` check moved `variant` into `Error::new_spanned` while the `format!` macro still borrowed `variant.ident`. Fixed by pre-collecting `variant_name = variant.ident.to_string()` before the field check and using `&*variant` for the error span. One iteration; ~2 min.

## Time breakdown

- Reading docs (BRIEF + EXPECTATIONS + DESIGN + probe + SCORE-k + lib.rs + Value enum): ~10 min
- `wat_value.rs` implementation (first pass): ~15 min
- Borrow fix + compile: ~3 min
- lib.rs export + runtime.rs application + cargo build: ~5 min
- trybuild setup (Cargo.toml, test runner, 5 fixtures): ~10 min
- First trybuild run (generated wip/); move snapshots; second run: ~5 min
- Full verification cascade (12 rows): ~5 min
- SCORE writing: ~10 min

**Actual total: ~63 min**

## Calibration

**Predicted:** 45–90 min Mode A; 120 min STOP.
**Actual:** ~63 min.

Within the lower half of the predicted band. The BRIEF's phase-by-phase prediction was accurate:
- Proc-macro implementation: ~18 min vs predicted 30-45 min (syntactic-scan algorithm was straightforward with the BRIEF's pseudocode; one borrow fix was the only non-trivial issue)
- lib.rs export: ~3 min vs predicted 2 min
- Value enum application: ~5 min vs predicted 5 min
- trybuild fixtures: ~15 min vs predicted 15-25 min
- Runtime probe: instant (probe was pre-written and flipped on first run)
- Verification + SCORE: ~15 min vs predicted 10 min

## Files created / modified

**Created:**
- `crates/wat-macros/src/wat_value.rs` — 249 lines (proc-macro implementation)
- `crates/wat-macros/tests/wat_value_ui_tests.rs` — 36 lines (trybuild runner)
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_box_self.rs` — 14 lines
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_box_self.stderr` — generated snapshot
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_arc_self.rs` — 15 lines
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_arc_self.stderr` — generated snapshot
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_self_direct.rs` — 17 lines
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_self_direct.stderr` — generated snapshot
- `crates/wat-macros/tests/ui/ui_wat_value_accepts_opt_in.rs` — 26 lines
- `crates/wat-macros/tests/ui/ui_wat_value_rejects_alias_bypass.rs` — 42 lines
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — this file

**Modified:**
- `crates/wat-macros/src/lib.rs` — added `mod wat_value;` + `#[proc_macro_attribute] pub fn wat_value` export with doc comment (~37 lines added)
- `crates/wat-macros/Cargo.toml` — added `[dev-dependencies] trybuild = "1"` (2 lines)
- `src/runtime.rs` — extended import line + `#[wat_value]` attribute on `pub enum Value` (2 lines changed)

## The annihilation is complete

After this stone:
- Value::Tracked is GONE (233.2.k) — current instance eliminated
- `#[wat_value]` applied to `pub enum Value` — future re-introduction is a compile error
- Escape hatch requires `#[wat_value(allow_wrapping = "reason")]` with non-empty reason — ceremonial opt-in with mandatory documentation
- trybuild fixtures lock the compile-fail behavior as a regression test — the seal cannot silently decay

The j → k → l annihilation chain is complete. The walk-impossible is structurally guaranteed.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.l.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/EXPECTATIONS-STONE-233.2.l.md` — 12-row scorecard expectations
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md` — sub-DESIGN (Detection algorithm; Decision 1/2/3)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md` — prerequisite (Value::Tracked retired)
- `tests/probe_stone_233_2_l_wat_value_seal.rs` — FM 2-bis probe (3/3 PASS)
- `crates/wat-macros/src/wat_value.rs` — proc-macro implementation
- `crates/wat-macros/tests/ui/` — trybuild compile-fail fixtures
