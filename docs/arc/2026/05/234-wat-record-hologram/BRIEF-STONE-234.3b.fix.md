# BRIEF — Stone 234.3b.fix — `RuntimeError::UnknownField` variant

**Status:** READY TO SPAWN.

## What to do

Mint `RuntimeError::UnknownField` variant; migrate `eval_record_assoc` to use it instead of `MalformedForm`. Honors the discipline that every error has its own variant (no catch-all stuffing).

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3b.fix.md` — sub-DESIGN with locked decisions
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3b.fix.md` — scorecard
3. `src/runtime.rs::eval_record_assoc` — the migration target (currently uses MalformedForm)
4. `src/runtime.rs:1897` — RuntimeError enum (where the new variant goes)
5. `src/runtime_error_edn.rs:48+` — EDN serializer (where the new arm goes)
6. `src/runtime_error_edn.rs:~299` — variant-name → string lookup (also add arm)

## Variant shape

```rust
UnknownField {
    record_class: String,   // bare FQDN, e.g. "myapp::Voltage"
    field: String,          // bare field-name attempted
    available: Vec<String>, // known field names on the record
    span: Span,
}
```

## Implementation

1. Add variant to `RuntimeError` enum in `src/runtime.rs`
2. Add EDN serializer arm in `src/runtime_error_edn.rs` (per existing pattern; map with 4 entries)
3. Add variant-name match arm: `RuntimeError::UnknownField { .. } => "UnknownField"`
4. Update `eval_record_assoc` in `src/runtime.rs`:
   - During the holon_form walk, COLLECT field names into a `Vec<String>` (probably already iterating)
   - When key not found: construct `RuntimeError::UnknownField { record_class, field, available, span }` instead of `MalformedForm`
5. Substrate-as-teacher: any other exhaustive match site on RuntimeError surfaces compile errors; add arms uniformly

## Discipline

- `src/runtime.rs` + `src/runtime_error_edn.rs` + ANY OTHER files with exhaustive RuntimeError matches (sonnet adds arms per compile errors; no unrelated work)
- DO NOT touch: probe files, wat/Record.wat, prior SCOREs, holon-rs (STOP-4)
- DO NOT commit (orchestrator commits)
- DO NOT add other error variants beyond UnknownField (STOP-6)
- DO NOT refactor existing variants (STOP-6)

## STOP triggers — all REJECTION

- STOP-1 unexpected compile errors not tracing to UnknownField
- STOP-2 lib baseline < 827
- STOP-3 45 min elapsed
- STOP-4 holon-rs touched
- STOP-5 Rust changes in files NOT involving RuntimeError exhaustive match (= must trace to the variant addition)
- STOP-6 scope creep
- STOP-7 Stone 234.3b probe regresses
- STOP-8 any prior arc 234 regression regresses
- STOP-9 clippy > 54

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3b.fix.md` (NEW). 11-row scorecard verbatim + implementation surface + which files needed new arms + cascade depth + time.
