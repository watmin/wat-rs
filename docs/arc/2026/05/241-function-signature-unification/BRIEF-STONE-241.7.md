# BRIEF — Stone 241.7 — mint `:wat::runtime::metadata-of`; Phase 2 closes

You are sonnet. Mint the reflection verb that reads `SymbolTable.binding_metadata` (Stone 241.6's storage). Mirror `eval_body_of` at `src/runtime.rs:13660` — sibling pattern.

## What to do

### S1 — Mint `eval_metadata_of` next to `eval_body_of` (runtime.rs ~13715)

Pattern (mirror body-of):

```rust
/// `(:wat::runtime::metadata-of <name :keyword>) -> :Option<HashMap<Keyword, HolonAST>>`
///
/// Stone 241.7. Returns the binding's metadata-map as Option:
/// - Some(map) when metadata was attached at def time (Stone 241.6 storage)
/// - None when binding exists but no metadata
/// - None when binding doesn't exist
fn eval_metadata_of(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::runtime::metadata-of";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: Span::unknown(),
        });
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    let name = match name_from_keyword_or_fn(&v) {
        Some(n) => n,
        None => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::keyword or named function",
                got: ValueSnapshot::of(&v),
                span: args[0].span().clone(),
            });
        }
    };
    
    match sym.binding_metadata.get(&name) {
        Some(meta) if !meta.is_empty() => {
            // Build Value::HashMap<Keyword, HolonAST> from meta entries.
            // Sonnet: find the Value::HashMap constructor pattern + key/value
            // conversion (String -> Value::Keyword; WatAST -> HolonAST via
            // watast_to_holon). Then wrap in Option::Some.
            let map_val = /* construct HashMap value */;
            Ok(Value::Option(Arc::new(Some(map_val))))
        }
        _ => Ok(Value::Option(Arc::new(None))),
    }
}
```

### S2 — Dispatch entry near runtime.rs:5565

Where `:wat::runtime::body-of` dispatches to `eval_body_of`, add sibling:

```rust
":wat::runtime::metadata-of" => eval_metadata_of(args, list_span, env, sym),
```

### S3 — HashMap value construction

Investigate via `grep "Value::HashMap" src/runtime.rs` for existing construction patterns. Likely a `Value::HashMap(Arc<HolonMap>)` or `Value::HashMap(...)` with insert methods. STOP-6 if requires more than ~15 lines (e.g., new HolonMap ctor).

For each `(key_string, watast_value)` in `meta`:
- key: `Value::Keyword(key_string.clone())` — the String already includes `:` prefix
- value: `Value::holon__HolonAST(Arc::new(watast_to_holon(&watast_value)))`

Insert into the HashMap value being built. Return `Value::Option(Arc::new(Some(map_val)))`.

## Discipline

- Touch ONLY `src/runtime.rs` (verb mint + dispatch entry) — NO other files
- `src/argspec/*` + `src/check.rs` + `src/lib.rs` UNCHANGED
- Stone 241.1-241.6 probes UNCHANGED at PASS counts
- No new ArgSpecError / ClauseFailureReason variants
- No new SymbolTable fields (Stone 241.6 added binding_metadata; Stone 241.7 just reads it)
- Reuse `name_from_keyword_or_fn` (body-of's helper)
- Reuse `watast_to_holon` (body-of's converter)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.7.md` — this
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.7.md` — D1-D8 + T1-T10 + STOP
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.6.md` — storage shipped
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` § Reflection — verb lock + return shape
6. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 5560-5570 (body-of dispatch) + 13651-13715 (eval_body_of)
7. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone7_metadata_of_reflection.rs` — 5-contract FM 2-bis probe

## Implementation sketch

1. Read body-of pattern + Stone 241.6 SCORE (binding_metadata structure)
2. Baseline: lib 834; Stone 241.7 probe expected to FAIL at HEAD (verb doesn't exist)
3. Grep `Value::HashMap` for the construction pattern
4. Implement `eval_metadata_of` mirroring body-of
5. Add dispatch entry near body-of
6. Run Stone 241.7 probe; iterate to 5/5
7. Verify lib + all Stone 241.x probes preserved
8. Write SCORE doc
9. DO NOT COMMIT

## STOP triggers

1. Compile errors not traced to verb mint
2. Lib < 834
3. 30 min elapsed
4. holon-rs touched
5. Files outside `src/runtime.rs` + `tests/probe_arc241_stone7_*` + SCORE doc. `src/argspec/*` + `src/check.rs` + `src/lib.rs` MUST stay unchanged.
6. Scope creep: HARD CUTs; HashMap construction > 15 lines (STOP-6 surface); new SymbolTable fields; modifying body-of or other reflection verbs
7. Stone 241.7 probe < 5/5
8. Stone 241.x / arc 237/238 probes regress
9. Clippy > 902

## SCORE doc spec

Mirror SCORE-STONE-241.6.md shape (no vigilia; legacy flat substrate). Include 10-row scorecard + 4-row structural + verb body verbatim + HashMap construction approach + cascade depth + PHASE 2 CLOSES inscription.

## Post-strike

Return one-paragraph status. Phase 2 closes with this stone. Phase 3 HARD CUTs (241.8 defstruct; 241.9 defenum; 241.10 define ⇒ defn) open next.
