# BRIEF — Arc 234 Stone 234.5 — `:wat::holon::*` auto-dispatch on `Value::wat__Record`

**Status:** READY TO SPAWN (2026-05-24).

**Predecessor SCOREs:** `SCORE-STONE-234.2b.md` (the macro consumer), `SCORE-STONE-234.2a-CORRECTION.md` (custom-handler precedent for check.rs), `SCORE-STONE-234.1.md` (variant + Eq/Hash/Display).

---

## What to do

Extend 5 substrate verbs in `:wat::holon::*` to auto-dispatch on `Value::wat__Record` — when called with a record arg, unwrap to its pre-built `holon_form` field, eliminating the user-facing `(:wat::core::record->holon r)` conversion call.

Verbs:
1. `:wat::holon::to-holon` (the polymorphic bridge)
2. `:wat::holon::Bind` (constructor)
3. `:wat::holon::Bundle` (constructor)
4. `:wat::holon::cosine` (VSA-proof verb)
5. `:wat::holon::extract-classifier` (algebraic type-extraction)

Two files change: `src/runtime.rs` + `src/check.rs`. Nothing else.

This is the stone that proves the hologram property is REAL — externally observable via VSA verbs accepting records without conversion.

## Read these in order

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.5.md`** — sub-DESIGN with 9 locked decisions + 8 trap-doors. THE LOAD-BEARING ARTIFACT.

2. **`docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.5.md`** — 11-row scorecard.

3. **`tests/probe_arc234_stone5_holon_auto_dispatch.rs`** — the load-bearing test (6/6 FAIL initial; goal 6/6 PASS).

4. **`src/runtime.rs::to_holon_inner`** (line ~15198) — polymorphic UP body; add `Value::wat__Record` arm.

5. **`src/runtime.rs::eval_algebra_bind`** (line ~16286), **`eval_algebra_bundle`** (line ~16341), **`eval_algebra_cosine`** (line ~17153) — the verb impls; identify each HolonAST-arg extraction site and thread the centralized helper.

6. **`src/runtime.rs::eval_extract_classifier`** — locate via `grep -n "fn eval_extract_classifier" src/runtime.rs`.

7. **`src/check.rs` line 5616** — cosine TypeScheme registration area; the broader patterns for HolonAST-typed primitives live nearby.

8. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md`** — `infer_record_of` custom-handler precedent for check.rs broadening.

## Implementation guidance

### Runtime side

**Pattern: centralized helper.** Add a fn `value_to_holon_ast` (sonnet names it) that maps:
- `Value::holon__HolonAST(h)` → `(*h).clone()` (existing case)
- `Value::wat__Record { holon_form, .. }` → `(*holon_form).clone()` (NEW)
- Other → `Err(RuntimeError::TypeMismatch { expected: "HolonAST or wat::Record", got: ..., span })`

Then each of the 4 algebra verbs (Bind, Bundle, cosine, extract-classifier) replaces its HolonAST-arg-extraction logic with a call to this helper. The 5th verb — `to-holon` — adds the `Value::wat__Record` arm directly to `to_holon_inner`.

For `Bundle`: the arg is a Vec of children. Iterate + apply the helper to each element.

For `Bind`: two args (left, right). Each goes through the helper.

For `cosine`: investigate `pair_values_to_vectors` (called at runtime.rs line 17169). If it already has a HolonAST-extraction path, thread the helper into it. If not: the helper is called BEFORE `pair_values_to_vectors` to normalize args.

### Check.rs side

5 TypeSchemes broaden to accept `:wat::Record` in `:wat::holon::HolonAST` positions.

Two viable approaches:
- **Custom handlers per verb** (precedent: `infer_record_of` for `:wat::Record::of`). Each verb gets its own custom inference fn that accepts both types in HolonAST positions.
- **Centralized accept-helper** in the inference dispatcher. A single helper recognizes "this param wants HolonAST; the arg is wat::Record" as valid; substitutes the type in unification.

Sonnet picks based on what composes cleanly with existing patterns. The Stone 234.2a-CORRECTION pattern is the closest precedent — mirror it for one verb to validate the approach, then extend to the other 4.

For TypeScheme positions that currently say `TypeExpr::Path(":wat::holon::HolonAST".into())` — the simplest fix may be a custom inference handler that accepts either path. Investigate.

## Discipline reminders

- **`src/runtime.rs` + `src/check.rs` ONLY** — STOP-5 fires on any other Rust change
- **NO modifications to `wat/Record.wat`** — the 234.2b macro is correct
- **NO modifications to existing probes** — only the new 234.5 probe is in scope; ALL prior probes must stay green
- **NO new verbs beyond the 5 named** — STOP-6 fires; defer broader sweep to Stone 234.6
- **NO class_fqdn checks inside VSA verb bodies** — D5 explicitly rejects; that's Stone 234.2c
- **NO touching holon-rs** — STOP-4
- **HARD CUT — no escape-hatch aliases** — D8 (records flow through; users compose directly)

## What to commit

ONE new file + TWO modified files:
1. `src/runtime.rs` (MODIFIED — helper fn + verb threadings + `to_holon_inner` arm)
2. `src/check.rs` (MODIFIED — TypeScheme broadenings or custom handlers)
3. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` (NEW — your SCORE)

DO NOT COMMIT. The orchestrator commits after independent verification + atomic SCORE inclusion.

## How you'll be scored

Per `EXPECTATIONS-STONE-234.5.md`. 11-row scorecard; binding command per row. Mode A target: 11/11 PASS.

LOAD-BEARING row: row 2 — the new probe flipping 6/6 FAIL → 6/6 PASS.

SECONDARY LOAD-BEARING rows: rows 3-7 (all arc 234 regression guards stay green; the substrate extension must not regress the macro consumer or earlier stones).

Per FM 9: rows are claims; commands are proof.

The SCORE doc captures:
- 11-row scorecard with verbatim command outputs
- Implementation pattern chosen (centralized helper vs per-verb threading; custom handler vs broadened TypeScheme)
- Runtime: helper fn line count + per-verb threading line count + `to_holon_inner` arm
- Check.rs: TypeScheme/handler approach + line count
- Cascade depth (compile rounds + iteration cycles)
- Time breakdown
- Calibration delta (60-90 target; 120 STOP)
- Rank-up evidence — Stone 234.2a-CORRECTION's `infer_record_of` precedent effectiveness
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.5.md` — sub-DESIGN (load-bearing)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.5.md` — paired EXPECTATIONS + scorecard
- `tests/probe_arc234_stone5_holon_auto_dispatch.rs` — the FM 2-bis probe (6/6 FAIL verified)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — macro consumer that benefits from this
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (line 290: VSA integration intent)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_no_known_defect_left_unfixed.md` — STOP triggers are rejection, not deferral
