# BRIEF — Arc 234 Stone 234.3b — `:wat::Record/assoc` substrate primitive

**Status:** READY TO SPAWN.

**Predecessors:** SCORE-STONE-234.3a (field-name extraction machinery to reuse), SCORE-STONE-234.5 (coerce_to_holon_ast helper for new value → HolonAST conversion), SCORE-STONE-234.2a (substrate primitive registration precedent).

## What to do

Mint `:wat::Record/assoc` substrate primitive. 3-arity: record + key-keyword + value → new record (immutable). Two files: `src/runtime.rs` (new eval fn + dispatch arm) + `src/check.rs` (new TypeScheme).

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.3b.md` (this)
2. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3b.md` — locked decisions + trap-doors
3. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.3b.md` — 11-row scorecard
4. `tests/probe_arc234_stone3b_record_assoc.rs` — load-bearing test (initial 1/6 PASS via lenient match; goal 6/6 PASS)
5. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3a.md` — read-verb predecessor; eval_record_to_map's field-walking pattern to reuse
6. `src/runtime.rs::eval_record_to_map` — the 234.3a fn that already walks holon_form for field names
7. `src/runtime.rs::eval_hashmap_assoc` (line ~9970) + `hashmap_assoc_inner` (~9848) — Arc-functional precedent for assoc
8. `src/runtime.rs::coerce_to_holon_ast` (added by 234.5) — converts Value → HolonAST for the new value's holon_form representation
9. `src/runtime.rs::eval_record_field_at` (line ~14579) — Value::wat__Record destructure pattern

## Implementation

```rust
fn eval_record_assoc(args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable)
    -> Result<Value, RuntimeError>
{
    if args.len() != 3 { return arity error; }
    let record = eval_inner(&args[0], env, sym)?.value_owned();
    let key    = eval_inner(&args[1], env, sym)?.value_owned();
    let new_val = eval_inner(&args[2], env, sym)?.value_owned();

    match record {
        Value::wat__Record { class_fqdn, struct_form, holon_form } => {
            // 1. Extract key's bare name (strip leading colon per Stone 234.2a SCORE D5)
            let key_name = match key {
                Value::wat__core__keyword(k) => k.strip_prefix(':').unwrap_or(&k).to_string(),
                _ => TypeMismatch error,
            };
            // 2. Walk holon_form to find field-name + index (same pattern as eval_record_to_map)
            //    holon_form = Bind(Atom(class), Bundle(field-binds))
            //    For each field-bind at index i: extract its name; if match key_name, capture i
            // 3. If no match: UnknownField error with available names list
            // 4. Type check: struct_form[i].type_name() == new_val.type_name(); else TypeMismatch
            // 5. Build new struct_form: clone Vec, replace[i] with new_val
            // 6. Build new holon_form: clone outer Bind; replace its Bundle's child[i]
            //    with Bind(Atom(String(name)), coerce_to_holon_ast(new_val))
            // 7. Return Value::wat__Record { class_fqdn, struct_form: new, holon_form: new }
        }
        other => TypeMismatch { expected: ":wat::Record", got: ValueSnapshot::of(&other), .. }
    }
}
```

Dispatch arm: `":wat::Record/assoc" => eval_record_assoc(args, list_span, env, sym),`

check.rs TypeScheme:
```rust
env.register(":wat::Record/assoc".into(), TypeScheme {
    type_params: vec!["T".into()],
    params: vec![record_ty(), keyword_ty(), t_var()],
    ret: record_ty(),
    rest_param_type: None,
});
```

## Discipline

- src/runtime.rs + src/check.rs ONLY (STOP-5)
- NO polymorphic :wat::core::assoc upgrade (deferred per scope)
- NO variadic key-value (compose handles it)
- NO touching wat/Record.wat, any probe, prior SCOREs
- NO holon-rs touching (STOP-4)

## STOP triggers

All REJECTION criteria. No deferral slots. STOP-3 at 90 min wall-clock.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3b.md` (NEW). Capture: 11-row scorecard verbatim; implementation surface line count; HolonAST rebuild approach (Bundle child replacement pattern); UnknownField error variant (existing or new); cascade depth; honest deltas.

Return when 11/11 PASS captured, OR when STOP fires with diagnostic.
