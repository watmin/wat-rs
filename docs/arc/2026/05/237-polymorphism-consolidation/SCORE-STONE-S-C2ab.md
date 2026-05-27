# SCORE — Stone S-C.2ab — field names → RecordDef + re-route name-access off holon_form

**Date:** 2026-05-27
**Status:** COMPLETE — build clean; 827/0 lib baseline; all regression probes green; parity confirmed.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors (pre-existing warnings ceiling) |
| 2 | **Lib baseline 827/0** (LOAD-BEARING) | `cargo test --release --lib -p wat 2>&1 \| tail -5` | `827 passed; 0 failed` |
| 3 | keyword-access parity | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 4 | assoc parity | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 5 | S-A1 regression | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 6 | S-B.1 regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 7 | S-B.2 regression | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 8 | arc227 stone2 regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -5` | `35 passed; 0 failed` |
| 9 | src/ + named tests only | STOP-2/STOP-5 check | confirmed — zero files outside `src/` + `wat/Record.wat` + the 2 named test files |

---

## RecordDef change (`src/types.rs`)

### New field

`RecordDef` gains `field_names: Vec<String>` (declaration order). Ruby model: the CLASS defines the attrs; the instance (`struct_form`) holds the values positionally.

```rust
pub struct RecordDef {
    pub name: String,
    pub parent: String,
    /// Field names in declaration order. Empty for zero-field records.
    pub field_names: Vec<String>,
}
```

`PartialEq`/`Eq` derived — idempotent re-registration check works correctly.

---

## `recordtype` change — HARD CUT to 3-arg (`src/types.rs:2344`)

### New arity

`(:wat::core::recordtype :Name :Parent [field-name-strings])` — 3 required args. A 2-arg call is rejected with a teaching error naming the missing `[field-names]` arg.

### Parse path

`parse_recordtype` extracts `args[2]` as `WatAST::Vector`. Each element must be `WatAST::StringLit(name, _)` → pushed into `field_names: Vec<String>`. Non-string elements produce `MalformedDecl` with diagnostic. Zero-field records pass `[]` → `field_names: vec![]`.

---

## `:wat::Record::def` macro change (`wat/Record.wat`)

### New emission

The macro now emits the 3-arg `recordtype` form, splicing field-name strings in declaration order:

```wat
(:wat::core::recordtype ~fqdn :wat::Record
  [~@(let [fields-h ... nf ... children ...]
        (map (range 0 nf)
             (fn [fi] (let [idx ... name-h ... name-s ...]
                        (to-wat (to-holon name-s))))))])
```

`name-s` is `(keyword/to-string (from-holon name-h))` — the bare field name string already extracted by the existing accessor-emission loop. `(to-wat (to-holon name-s))` converts `Value::String` → `HolonAST::String` → `WatAST::StringLit`, which `parse_recordtype` accepts as a string literal in the vector.

The existing struct_form symbols, holon-form field-bind, and accessor emission loops are **unchanged** — the field-name extraction is reused (new splice, same pattern, third loop added in parallel).

---

## The 3 re-routed name→index sites (`src/runtime.rs`)

### Site 1 — `keyword_accessor_record` (formerly line 6432)

**Before:** walked `holon_form` Bundle children to find `bare_name`; required `holon_form: Arc<HolonAST>` parameter.

**After:** looks up `RecordDef` in the TypeEnv via `format!(":{}", class_fqdn)` + `sym.types()`; calls `field_names.iter().position(|n| n == bare_name)`. New signature drops `holon_form` parameter, gains `sym: &SymbolTable`.

```rust
fn keyword_accessor_record(
    bare_name: &str,
    class_fqdn: Arc<String>,
    struct_form: Arc<Vec<Value>>,
    sym: &SymbolTable,            // ← replaces holon_form
    list_span: &Span,
) -> Result<Value, RuntimeError>
```

### Site 2 — `eval_record_to_map` (`:wat::core::record->map`, formerly line 16663)

**Before:** walked `holon_form` to build `HashMap<keyword, value>`.

**After:** looks up `RecordDef.field_names`, iterates over them with index, reads `struct_form[i]` directly. No holon_form touched.

### Site 3 — `eval_record_assoc` name lookup (`:wat::Record/assoc`, formerly line 16773)

**Before:** walked `holon_form` to find `key_name` → `field_index`.

**After:** looks up `RecordDef.field_names`; uses `.iter().position(|n| n == &key_name)`. `field_binds` is still extracted from `holon_form` (hoisted below the name-lookup block) — needed for the parity holon_form REBUILD (PARITY invariant: holonic `assoc` must rebuild BOTH `struct_form` AND `holon_form`). Only the name→index *source* changed; the rebuild is untouched.

---

## recordtype-caller arity updates

| File | Change |
|------|--------|
| `wat/Record.wat` | macro emits `(:wat::core::recordtype ~fqdn :wat::Record [~@name-strs])` (3-arg) |
| `tests/probe_arc237_sB1_recordtype.rs` | PRELUDE: 2-arg → 3-arg (`[]` both); probe_06 inline src: 2-arg → 3-arg |
| `tests/probe_arc237_sA1_assignable.rs` | probe_05 inline: `recordtype :my::Special :my::Circle` → `recordtype :my::Special :my::Circle []` |

`probe_arc237_sB2_defrecord_recordtype.rs` and `probe_arc227_stone2_defrecord.rs` drive the `defrecord` surface only — no direct `recordtype` calls; untouched, passed 5/5 and 35/35.

---

## Parity verification

Both `probe_arc234_stone3c_keyword_accessor` and `probe_arc234_stone3b_record_assoc` use multi-field records (`myapp::Triple` with 3 fields `a/b/c`) and verify per-field values by name. Probe 2 of each confirms name-order: `(:b t)` on `Triple(7, "hello", true)` → `"hello"` (index 1, not 0). These are unchanged at 6/6 — **parity confirmed; re-route gives identical answers**.

---

## Honest deltas

### No cascade rounds

One pass of changes; `cargo build --release -p wat` → 0 errors on first attempt. All 7 probes + lib baseline green on first test run.

### Macro emission path

The `to-wat (to-holon name-s)` chain in the macro is the minimal honest path: `Value::String` → `HolonAST::String` (via `to_holon_inner`) → `WatAST::StringLit` (via line 18449 in `runtime.rs`). `WatAST::StringLit` is what `parse_recordtype`'s `WatAST::StringLit(s, _)` arm accepts. The chain is reversible and unambiguous.

### `field_binds` in assoc

The parity-rebuild in `eval_record_assoc` still needs `field_binds` from `holon_form` (to replace child at `field_index` in the new Bundle). It was hoisted below the name-lookup block with a comment marking the invariant. No structural change to the rebuild itself.

### `holon_form` removed from `keyword_accessor_record` callers

Three callers updated to `..` pattern: the keyword-as-accessor fall-through (~6381), the `LetBinding::HashDestructure` arm (~7790), and the match-destructure arm (~15817). All had `sym` already in scope.

### Files outside src/ + named test files — zero

`holon-rs` untouched. STOP-5 not triggered.

---

## Working tree on return

```
 M src/runtime.rs
 M src/types.rs
 M tests/probe_arc237_sA1_assignable.rs
 M tests/probe_arc237_sB1_recordtype.rs
 M wat/Record.wat
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C2ab.md
```

holon-rs untouched. DO NOT commit (orchestrator commits).
