# BRIEF — Stone 234.4 — let-binding hash-destructure

**Status:** READY TO SPAWN.

**Predecessors:** Arc 169 (struct-destructure), Stones 234.3a/3b/3c (record polymorphic verbs + keyword-as-accessor field-walking pattern).

## What to do

Add Clojure-style hash-destructure `{var :field var2 :field2 ...}` in let-binding position. Receiver-polymorphic over record/struct/HashMap. Three files:
- `src/parser.rs` — disambiguation + new AST node (or extended struct-destructure node)
- `src/check.rs` — binding-scope extension for the pattern
- `src/runtime.rs` — field extraction per receiver variant

NOT in scope: match-arm hash-destructure (deferred to named Stone 234.4.match).

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.4.md` — 10 locked decisions
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.4.md` — 11-row scorecard
3. `tests/probe_arc234_stone4_hash_destructure.rs` — load-bearing test (6/6 FAIL initial)
4. `src/parser.rs` line ~252 (BraceKind discriminator) + line ~540 (parse_struct_destructure_body)
5. `src/runtime.rs::destructure_tuple` (line 6656 — arc 169 precedent for binding extension)
6. `src/check.rs::extend_pair_scope_with_tuple_destructure` (line 4359 — binding-scope precedent)
7. Stone 234.3c's `keyword_accessor_record` / `keyword_accessor_struct` helpers (field-walking pattern)

## Implementation

### Parser (src/parser.rs)

Current discriminator (line ~252) checks ONLY first item. Extend to peek second item:

```rust
let kind = match (items.first(), items.get(1)) {
    (None, _) => BraceKind::MapLiteral,
    (Some(WatAST::Symbol(_, _)), Some(WatAST::Keyword(_, _))) => BraceKind::HashDestructure,
    (Some(WatAST::Symbol(_, _)), _) => BraceKind::StructDestructure,
    _ => BraceKind::MapLiteral,
};
```

Add `BraceKind::HashDestructure` arm calling `parse_hash_destructure_body` which:
- Validates even count (each var needs a keyword partner)
- Validates alternating Symbol/Keyword positions
- Builds AST node `HashDestructure { bindings: Vec<(String, String)>, span }` where bindings carry (var-name, bare-field-name) pairs

### Check (src/check.rs)

Add binding-scope extension that recognizes HashDestructure AST node. For each (var, field) pair: introduce var into the body's scope. Type per receiver-type-of-RHS — if known to be record/struct, use field's declared type; if HashMap or unknown, use `:wat::core::Option<T>` (HashMap arm always Option).

Receiver type validation: if RHS resolves to non-record/struct/HashMap type, TypeMismatch error at check time.

### Runtime (src/runtime.rs)

In let-binding evaluation, when binder is HashDestructure AST node:
1. Evaluate RHS once
2. Dispatch on Value variant:
   - `Value::wat__Record { struct_form, holon_form, class_fqdn }` → walk holon_form bundles; for each binding (var, field), find field by name; bind var to struct_form[i]. Missing → UnknownField (variant from 234.3b.fix).
   - `Value::Struct(sv)` → look up each field-name in sv.type_def; bind var to sv.fields[i]. Missing → UnknownField.
   - `Value::wat__std__HashMap(map)` → build keyword Value lookup key per binding; wrap result in Value::Option(Some/None); bind var.
   - Other → TypeMismatch.
3. Bindings live in let body's scope. Reuses arc 169 binding-extension pattern.

Reuse Stone 234.3c's `keyword_accessor_record` / `keyword_accessor_struct` helpers if their signatures fit (single-field lookup); destructure may need a multi-field-collection variant.

## Discipline

- src/parser.rs + src/check.rs + src/runtime.rs ONLY (STOP-5)
- DO NOT touch: wat sources, probes, prior SCORE docs, holon-rs (STOP-4)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT add match-arm support (STOP-6; Stone 234.4.match is named successor)
- DO NOT add per-class TypeDef registration (STOP-6; arc 232.1 future-lift)
- DO NOT add positional-bind form via brace (STOP-6; positional uses vec-form per arc 169)

## STOP triggers (REJECTION)

1. unexpected compile errors
2. lib baseline < 827
3. 150 min elapsed
4. holon-rs touched
5. Rust changes outside parser.rs + check.rs + runtime.rs
6. scope creep (match-arm, per-class TypeDef, positional-bind)
7. probe doesn't flip 6/6 PASS
8. 234.3c regression
9. any prior arc 234 regression
10. clippy > 54

If STOP fires: report; surface; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.md` (NEW). 11-row verbatim + which receivers shipped + cascade depth + time + honest deltas.

If receiver coverage proves heavy, DEFER specific arms with NAMED successor stones (e.g., "Stone 234.4.struct" if struct arm defers), NOT "future cleanup." Per the discipline lesson from 234.3b.fix.
