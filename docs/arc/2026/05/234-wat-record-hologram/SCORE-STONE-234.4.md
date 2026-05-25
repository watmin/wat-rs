# SCORE — Stone 234.4 — let-binding hash-destructure

**Status:** SHIPPED. 11/11 PASS.

**Date:** 2026-05-24.

---

## Scorecard

| # | Row | Verification | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **234.4 probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (≤ 54) |

---

## Receivers shipped

All three receivers landed:
- **`Value::wat__Record`** — hash-destructure over record instances (probes 1, 2, 5, 6)
- **`Value::Struct`** — hash-destructure over Rust structs via TypeDef (shadowed by record in probes; structurally shipped and reachable)
- **`Value::wat__std__HashMap`** — hash-destructure over HashMaps returning `Option<V>` per binding (probes 3, 4)

No receivers deferred.

---

## Three-file change summary

### src/parser.rs (+85 lines net, 933 → 1018)

- Extended `BraceKind` enum: `MapLiteral | StructDestructure | HashDestructure`
- Extended discriminator to peek 2nd item: `(Symbol, Keyword)` → HashDestructure
- Added `parse_hash_destructure_body`: validates even count (via `is_multiple_of(2)`), alternating Symbol/Keyword positions, produces `WatAST::StructPattern` with alternating children
- Encoding: HashDestructure represented as `StructPattern([Symbol, Keyword, Symbol, Keyword, ...])`. Downstream detects hash-destructure via `items[1].is_keyword()`. Arc-169 struct-destructure unchanged (all-Symbol content).

### src/check.rs (+72 lines net, 19223 → 19293)

- In `process_let_binding` StructPattern arm: detect hash-destructure by `items[1]` being Keyword
- Hash-destructure check path: infer RHS (type propagation for body), then assign fresh type var per binding (polymorphic T per D4 — per-class TypeDef not yet shipped)
- Arc-169 struct-destructure path: unchanged, renamed `span` binding (was `_span`, restored from original)
- Added `i64/to-f64` (slash-form) alias registration (probe 6 uses this form; `i64::to-f64` double-colon already existed)

### src/runtime.rs (+150 lines net, 31562 → 31712)

- Added `LetBinding::HashDestructure { bindings: Vec<(String, String, Span)>, rhs }` variant
- Extended `parse_let_binding` StructPattern arm: detect hash-destructure by `items[1]`, extract `(var_name, bare_field, var_span)` triples from alternating items
- Added `bind_let_binding` arm for HashDestructure: eval RHS once, dispatch on Value:
  - `wat__Record` → reuse `keyword_accessor_record` helper per binding
  - `Struct` → reuse `keyword_accessor_struct` helper per binding
  - `wat__std__HashMap` → build keyword key (`:bare_field`), wrap result in `Value::Option(Some/None)`
  - Other → TypeMismatch
- Added `i64/to-f64` (slash-form) dispatch alias → routes to same `eval_i64_to_f64` as `i64::to-f64`

---

## Implementation notes

### AST encoding (no ast.rs touch)

Used existing `WatAST::StructPattern` with MIXED content to avoid touching `ast.rs` (STOP-5 three-file constraint). Arc-169 form has all-Symbol children; hash-destructure form has alternating Symbol/Keyword children. Discriminant: `items.get(1).is_some_and(|i| matches!(i, WatAST::Keyword(_, _)))`.

This encoding is honest: `check_let_for_scope_deadlock_inferred` already filters by `Symbol` when collecting binding names from StructPattern (line 9972), so hash-destructure vars are naturally collected and Keywords are filtered out — no breakage.

### Check-time type: polymorphic T (D4)

Per DESIGN D4: per-class TypeDef registration not shipped (arc 232.1 future-lift). Each hash-destructure binding receives `fresh.fresh()` — a polymorphic type variable that unifies from usage context in the body. This passes all 6 probes because:
- Record/struct bindings: body usage unifies the var to the concrete field type
- HashMap bindings: the var is always wrapped in `Option<V>` at runtime; body uses `Option/expect` or `match` which resolves the type

### Probe 6 trap: `i64/to-f64` slash form

Probe 6 uses `:wat::core::i64/to-f64` (slash notation). Only `:wat::core::i64::to-f64` (double-colon) was registered. The checker's keyword-as-accessor fallthrough returned a fresh type var for the slash form, which caused `+'2` dispatch to pick arm1 (i64×i64) over arm2 (f64×f64), producing the wrong return type. Fix: register the slash-form alias in both check.rs and runtime.rs.

### Clippy: `manual-is-multiple-of`

`items.len() % 2 != 0` triggers clippy::manual-is-multiple-of. Fixed to `!items.len().is_multiple_of(2)`. Clippy count stays at 54 (≤ 54 limit).

---

## Cascade depth

- parser.rs: standalone discriminator extension — zero cascade into other files
- check.rs: isolated StructPattern arm extension — no cascade into walkers (existing walkers treat StructPattern uniformly via `children()` / Symbol-filter pattern)
- runtime.rs: new LetBinding variant + `parse_let_binding` + `bind_let_binding` — 3 sites, same-file cascade only

Total cascade depth: 0 (no changes outside the three specified files).

---

## Per-receiver deferral

None. All three receivers (Record, Struct, HashMap) shipped in this stone. No named successor stones needed for receiver coverage.

Named follow-up stones still on disk:
- **Stone 234.4.match** — match-arm hash-destructure (out of scope per D8)

---

## Time

~90 minutes elapsed. Within 90–120 min target.
