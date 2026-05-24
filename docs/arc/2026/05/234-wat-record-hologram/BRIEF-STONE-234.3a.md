# BRIEF — Arc 234 Stone 234.3a — read verbs: `:wat::core::record?` + `:wat::core::record->map`

**Status:** READY TO SPAWN (2026-05-24).

**Predecessor SCOREs:** `SCORE-STONE-234.2c.md` (records are class-safe), `SCORE-STONE-234.5.md` (VSA integration; holon_form auto-dispatch precedent), `SCORE-STONE-234.2a-CORRECTION.md` (custom-handler precedent).

---

## What to do

Mint TWO new substrate primitives at `:wat::core::*`:

1. **`:wat::core::record?`** — polymorphic predicate; true iff input is `Value::wat__Record`
2. **`:wat::core::record->map`** — extract a `HashMap<:wat::core::keyword, :T>` from a record (field-names + corresponding typed values)

Both pure read verbs. Foundation for 234.3b (assoc polymorphic record arm) and 234.3c (keyword-as-accessor fall-through), which reuse the field-name extraction machinery established here.

TWO files change: `src/runtime.rs` (new eval fns + dispatch arms) + `src/check.rs` (new TypeScheme registrations OR custom handlers).

## Read these in order

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3a.md`** — sub-DESIGN with 12 locked decisions + 8 trap-doors. LOAD-BEARING.

2. **`docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3a.md`** — 11-row scorecard.

3. **`tests/probe_arc234_stone3a_record_read_verbs.rs`** — load-bearing test (6/6 FAIL initial; goal 6/6 PASS).

4. **`src/runtime.rs::eval_record_field_at`** (line ~14579) — Value::wat__Record pattern-match reference; positional access from struct_form.

5. **`src/runtime.rs::to_holon_inner`** (line ~15198) — polymorphic UP dispatch; Stone 234.5 added a `Value::wat__Record` arm here. Reference for the Arc<HolonAST> unwrap pattern.

6. **`src/runtime.rs::eval_bundle_children`** + **`eval_bind_left`** + **`eval_bind_right`** — HolonAST traversal helpers (locate via grep). Use these for walking holon_form's outer Bind → inner Bundle → field-Binds chain.

7. **`src/runtime.rs::eval_hashmap_assoc`** (line 9965) — HashMap arm of assoc; precedent for HashMap construction pattern (you'll need to build a HashMap in `record->map`).

8. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md`** — custom-handler precedent for check.rs if polymorphic-T inference needs special handling.

## Implementation guidance

### Runtime — `eval_record_q`

Trivial pattern match:

```rust
fn eval_record_q(args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable)
    -> Result<Value, RuntimeError>
{
    if args.len() != 1 { return arity error; }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::bool(matches!(v, Value::wat__Record { .. })))
}
```

Dispatch arm in primary dispatcher: `":wat::core::record?" => eval_record_q(args, list_span, env, sym),`

### Runtime — `eval_record_to_map`

Walk the record's `holon_form`:

```rust
fn eval_record_to_map(args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable)
    -> Result<Value, RuntimeError>
{
    if args.len() != 1 { return arity error; }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    match v {
        Value::wat__Record { struct_form, holon_form, .. } => {
            // holon_form is Arc<HolonAST> = Bind(Atom(class), Bundle(field-binds))
            // Extract Bundle's children (Vec<HolonAST>); each child is Bind(Atom(name), Atom(value))
            // For each child at index i:
            //   - field-name: extract leaf String from child.left (Atom containing String leaf)
            //   - field-value: struct_form[i] (positional match)
            // Build HashMap<Value::wat__core__keyword(":name"), struct_form[i].clone()>
            ...
            Ok(Value::wat__std__HashMap(Arc::new(map)))
        }
        other => Err(TypeMismatch { expected: ":wat::Record", got: ValueSnapshot::of(&other), ... })
    }
}
```

Investigate the HolonAST leaf extraction pattern by grepping for existing field-name-from-Bind extractors (likely in arc 230 / 225 substrate). The field-name leaves are produced by `(:wat::holon::Atom (:wat::holon::to-holon "<name>"))` at 234.2b macro expand time; per Stone 234.2c + 234.5 precedent, they're `HolonAST::Atom(HolonAST::String("<name>"))` or similar nested form.

For HashMap construction: per Stone 216 (Value::wat__std__HashMap uses `Arc<HashMap<Value, Value>>` post-216.5c). Build the map with `Value::wat__core__keyword(Arc::new(format!(":{}", name)))` keys and the typed struct_form values.

### Check.rs — `record?` TypeScheme

Polymorphic input; bool output:

```rust
env.register(":wat::core::record?".into(), TypeScheme {
    type_params: vec!["T".into()],
    params: vec![t_var()],
    ret: bool_ty(),
    rest_param_type: None,
});
```

Mirror `:wat::core::vector?` registration if it exists; grep for it.

### Check.rs — `record->map` TypeScheme

Input is `:wat::Record`; output is `HashMap<:wat::core::keyword, :T>`:

```rust
env.register(":wat::core::record->map".into(), TypeScheme {
    type_params: vec!["T".into()],
    params: vec![record_ty()],
    ret: TypeExpr::Parametric {
        head: "wat::core::HashMap".into(),
        args: vec![
            TypeExpr::Path(":wat::core::keyword".into()),
            t_var(),
        ],
    },
    rest_param_type: None,
});
```

If TypeScheme polymorphic-T doesn't compose cleanly with HashMap's typed K/V params, mint a custom handler per Stone 234.2a-CORRECTION's `infer_record_of` precedent.

## Discipline reminders

- **`src/runtime.rs` + `src/check.rs` ONLY** — STOP-5 fires on any other Rust change
- **NO modifications to `wat/Record.wat`** — the macro is correct
- **NO modifications to existing probes** — only the new 234.3a probe is in scope
- **NO `assoc` polymorphic arm** — Stone 234.3b scope; STOP-6 fires
- **NO keyword-as-accessor fall-through** — Stone 234.3c scope; STOP-6 fires
- **NO `:wat::core::record->holon` mint** — synonym with `:wat::holon::to-holon` (already polymorphic per 234.5); violates `feedback_wat_llm_first_design`
- **NO per-class accessor variants** — `:myapp::is-Voltage?` etc. are 234.2b macro-generated; 234.3a doesn't mint synonyms
- **NO `holon-rs` touching** — STOP-4

## What to commit

ONE new file + TWO modified files:
1. `src/runtime.rs` (MODIFIED — 2 new eval fns + 2 dispatch arms)
2. `src/check.rs` (MODIFIED — 2 TypeScheme registrations OR custom handlers)
3. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3a.md` (NEW — your SCORE)

DO NOT COMMIT. The orchestrator commits after independent verification.

## How you'll be scored

Per `EXPECTATIONS-STONE-234.3a.md`. 11-row scorecard; binding command per row. Mode A target: 11/11 PASS.

LOAD-BEARING row: row 2 — the new probe flipping 6/6 FAIL → 6/6 PASS.

The probe uses standard run_compute pattern + Rust-side Value match. Probe 5 uses `:wat::core::empty?` to verify empty HashMap; probes 3-4 use `:wat::core::get` to extract specific keys (HashMap polymorphism is established).

The SCORE doc captures:
- 11-row scorecard with verbatim command outputs
- Implementation pattern: how holon_form field-names were extracted (which existing helper); HashMap construction
- Per-fn line counts
- Cascade depth + iteration cycles
- Time breakdown
- Calibration delta (30-60 target; 75 STOP)
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3a.md` — sub-DESIGN (load-bearing)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3a.md` — paired EXPECTATIONS
- `tests/probe_arc234_stone3a_record_read_verbs.rs` — FM 2-bis probe (6/6 FAIL verified)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — precedent for holon_form access
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent
- `wat-rs/docs/WAT-CHEATSHEET.md` § 1 — colon rule (symbol-quote framing; parametric type args are bare)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
