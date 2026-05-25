# BRIEF — Stone 234.3c — keyword-as-accessor fall-through

**Status:** READY TO SPAWN.

**Predecessors:** SCORE-STONE-234.3a (record field-walking pattern), SCORE-STONE-234.3b.fix (UnknownField variant), SCORE-STONE-234.5 (centralized helper precedent).

## What to do

Add a fall-through arm in `dispatch_keyword_head_value` (`src/runtime.rs` line ~4866) that catches unknown-verb single-arg keyword-head calls and dispatches as field-access on receiver Value variant.

Three receivers: `Value::wat__Record` (record field), `Value::Struct` (struct field), `Value::wat__std__HashMap` (key lookup returning Option).

Two files: `src/runtime.rs` (intercept + 3 arms) + `src/check.rs` (polymorphic-T for unknown-verb-single-arg shape).

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3c.md` — sub-DESIGN; 11 locked decisions + 8 trap-doors
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3c.md` — 11-row scorecard
3. `tests/probe_arc234_stone3c_keyword_accessor.rs` — load-bearing test (1/6 PASS initial via lenient match; goal 6/6 PASS for right reasons)
4. `src/runtime.rs::dispatch_keyword_head_value` line 4866 — match-head dispatch (find the user-fn-lookup default arm at end; intercept before UnknownFunction return at line ~5909)
5. `src/runtime.rs::eval_record_to_map` (line ~14579 area, 234.3a) — field-name walking pattern to reuse for record arm
6. `src/runtime.rs::eval_record_assoc` (added 234.3b/3b.fix) — UnknownField construction pattern
7. `src/runtime.rs::hashmap_assoc_inner` (line ~9848) — HashMap Value-as-key precedent

## Implementation

Inside `dispatch_keyword_head_value`'s default arm (after user-fn lookup, just before line 5909 UnknownFunction return):

```rust
// Arc 234 Stone 234.3c — keyword-as-accessor fall-through.
// When head is an unknown verb AND args.len() == 1 AND receiver is
// {wat__Record, Struct, wat__std__HashMap}, dispatch as field accessor.
if args.len() == 1 {
    let receiver = eval_inner(&args[0], env, sym)?.value_owned();
    let bare_name = head.strip_prefix(':').unwrap_or(head);
    match receiver {
        Value::wat__Record { class_fqdn, struct_form, holon_form } => {
            // Walk holon_form Bundle children; find Bind with matching field-name
            // Return struct_form[i]; miss → UnknownField error
        }
        Value::Struct(sv) => {
            // Look up field-name in struct TypeDef field list
            // Return sv.fields[i]; miss → UnknownField error
        }
        Value::wat__std__HashMap(map) => {
            let key = Value::wat__core__keyword(Arc::new(head.to_string()));
            match map.get(&key) {
                Some(v) => Ok(Value::Option(Arc::new(Some(v.clone())))),
                None    => Ok(Value::Option(Arc::new(None))),
            }
        }
        _ => { /* fall through to UnknownFunction below */ }
    }
} else {
    /* fall through to UnknownFunction below */
}
```

check.rs side: extend the keyword-head dispatch (find via grep `dispatch_keyword_head\|infer_keyword_head`) to type-check unknown-verb-single-arg-with-record/struct/HashMap-receiver as polymorphic-T return. May need custom handler precedent (Stone 234.2a-CORRECTION's `infer_record_of`).

## Discipline

- src/runtime.rs + src/check.rs ONLY
- DO NOT touch: wat sources, probes, prior SCORE docs, holon-rs (STOP-4)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT add check-time narrowing (STOP-6; future arc 232.1 lift)
- DO NOT add receivers beyond {record, struct, HashMap}
- DO NOT mint alternative naming like `:wat::core::field-of`

## STOP triggers (REJECTION)

1. unexpected compile errors
2. lib baseline < 827
3. 120 min elapsed
4. holon-rs touched
5. Rust changes outside runtime.rs + check.rs
6. scope creep (check-time narrowing; extra receivers; per-class TypeDef)
7. probe doesn't flip to 5/6+ PASS
8. 234.3a regression
9. any prior arc 234 regression
10. clippy > 54

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3c.md` (NEW). 11-row verbatim + which receivers shipped (all 3? or struct deferred?) + check.rs approach + cascade depth + honest deltas.

## Note on probe 6 (struct)

If `:wat::core::struct` fixture proves heavy or substrate doesn't support the inline declaration pattern shown in the probe, document the struct arm as DEFERRED in SCORE and flip 5/6. Then ship a follow-up stone for the struct arm specifically. Acceptable — record + HashMap are the load-bearing 2/3.
