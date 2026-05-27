# SCORE — Stone S-C.2c — mint base `Value::wat__Record { class_fqdn, struct_form }`

**Date:** 2026-05-27
**Status:** COMPLETE — build clean; 828/0 lib baseline (827 prior + 1 new co-located unit test); all regression probes green; holon-op teaching error confirmed.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors (pre-existing warnings ceiling) |
| 2 | **Lib baseline 828/0** (LOAD-BEARING) | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `828 passed; 0 failed` |
| 3 | New probe 6/6 | `cargo test --release --test probe_arc237_sC2c_base_record 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 4 | S-C.2ab regression | `cargo test --release --test probe_arc237_sC2ab_field_order 2>&1 \| grep "test result"` | `5 passed; 0 failed` |
| 5 | S-A1 regression | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 6 | keyword-access parity | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 7 | assoc parity | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| grep "test result"` | `6 passed; 0 failed` |
| 8 | arc227 stone2 regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| grep "test result"` | `35 passed; 0 failed` |
| 9 | src/ + named tests only | STOP-2/STOP-5 check | confirmed — files touched: `src/runtime.rs`, `src/edn_shim.rs`, `src/closure_extract.rs`; probe on disk: `tests/probe_arc237_sC2c_base_record.rs`; zero holon-rs touches |

---

## The Variant

```rust
/// Stone S-C.2c — base (wat) record: the reduced flavor. EDN-restricted data
/// held in a positional `struct_form`; NO `holon_form`. Field NAMES live on
/// the class (`RecordDef.field_names`, S-C.2ab); name→index access rides that
/// path. Structural identity over `(class_fqdn, struct_form)`. Holon-ops are a
/// teaching error — base has no holon flavor (use a holonic record via
/// `:wat::holon::Record::def`). Unconstructed at the wat surface until S-C.3
/// mints `:wat::Record::def` → base.
wat__Record {
    /// Record class FQDN — e.g. `"my::Pt"` (no leading colon).
    class_fqdn: Arc<String>,
    /// Ordered field values in declaration order (fast Rust-side access).
    /// Structural identity lives here (with `class_fqdn`).
    struct_form: Arc<Vec<Value>>,
},
```

Inserted in `src/runtime.rs` immediately before `wat__core__clauses`, after `wat__holon__Record`.

---

## Bucket A — base-structural arms (NEW, distinct from holonic)

### PartialEq (`runtime.rs` PartialEq impl)

New arm beside holonic arm:
```rust
(Value::wat__Record { class_fqdn: a_cls, struct_form: sa },
 Value::wat__Record { class_fqdn: b_cls, struct_form: sb }) => {
    a_cls == b_cls && sa == sb
}
```
Cross pairs (base vs holonic) fall to the existing `_ => false`. Holonic arm untouched.

### Hash (`runtime.rs` Hash impl)

New arm with distinct discriminant tag:
```rust
Value::wat__Record { class_fqdn, struct_form } => {
    "wat__Record".hash(state);
    class_fqdn.hash(state);
    struct_form.hash(state);
}
```
Tag `"wat__Record"` vs `"wat__holon__Record"` prevents cross-variant hash collisions.

### assoc (`eval_record_assoc`, `runtime.rs`)

Base arm implemented as an early-return `if let` block before the holonic extraction:
- Extracts `(base_class, base_struct)` from `Value::wat__Record`.
- Looks up `RecordDef.field_names`, finds `field_index`.
- Rebuilds `struct_form` ONLY → returns `Value::wat__Record { .. }`.
- Holonic extraction + parity rebuild (both forms) is UNCHANGED below it.

---

## Bucket B — or-pattern sites (shared fields; identical behavior)

All ride `class_fqdn` and/or `struct_form`; `holon_form` is NOT accessed.

| Site | File | Change |
|------|------|--------|
| `type_name()` | `runtime.rs` | `Value::wat__holon__Record { .. } \| Value::wat__Record { .. } => "wat::Record"` |
| `declared_type_name()` | `runtime.rs` | `Value::wat__holon__Record { class_fqdn, .. } \| Value::wat__Record { class_fqdn, .. } => class_fqdn.to_string()` |
| `val_type_path` | `runtime.rs` | `Value::wat__holon__Record { .. } \| Value::wat__Record { .. } => ":wat::Record"` |
| keyword-accessor fall-through | `runtime.rs` ~6381 | or-pattern; calls `keyword_accessor_record` (variant-agnostic via RecordDef.field_names) |
| `LetBinding::HashDestructure` | `runtime.rs` ~7753 | or-pattern; calls `keyword_accessor_record` |
| match-destructure walk | `runtime.rs` ~15780 | or-pattern; calls `keyword_accessor_record` |
| conforms class_fqdn check | `runtime.rs` ~16157 | or-pattern; checks `class_fqdn.as_str() == stripped` |
| field-at positional | `runtime.rs` ~16543 | or-pattern; extracts `struct_form` |
| `record?` predicate | `runtime.rs` ~16605 | `matches!(v, Value::wat__holon__Record{..} \| Value::wat__Record{..})` |
| `record->map` | `runtime.rs` ~16643 | or-pattern; rides RecordDef.field_names + struct_form |
| `extract-classifier` | `runtime.rs` ~16940 | or-pattern; returns `Value::String(class_fqdn)` |
| `render_value` (Display) | `runtime.rs` ~20651 | or-pattern; renders `<class_fqdn{...}>` |
| `edn_shim::value_to_edn_with` | `edn_shim.rs` ~1697 | or-pattern; tagged-map EDN rendering |
| `closure_extract` | `closure_extract.rs` ~1725 | or-pattern; same `Internal` not-implemented error |

---

## Bucket C — holon-op teaching errors (base has no holon flavor)

Teaching message: `"base record \`<class>\` has no holon flavor; construct a holonic record (\`:wat::holon::Record::def\`) to use holon operations"`.

Uses `RuntimeError::MalformedForm` (carries `reason: String`) — `TypeMismatch.expected` is `&'static str` and cannot hold a dynamic class name.

| Site | File | Change |
|------|------|--------|
| `to_holon_inner` | `runtime.rs` ~17589 | new `Value::wat__Record { class_fqdn, .. }` arm → `Err(MalformedForm{..})` |
| `coerce_to_holon_ast` | `runtime.rs` ~19016 | new `Value::wat__Record { class_fqdn, .. }` arm → `Err(MalformedForm{..})` |
| normalize sites (two) | `runtime.rs` ~19368/72 | `Value::wat__Record` arm in each `let a = match` / `let b = match` → `Err(MalformedForm{..})` |

Co-located unit test in `src/runtime.rs` `#[cfg(test)] mod tests`:
```rust
#[test]
fn to_holon_inner_base_record_returns_err_with_teaching_message() { ... }
```
Verifies: `to_holon_inner(base, &span)` → `Err`, message contains `"base record"`, `"my::Pt"`, `"has no holon flavor"`, `":wat::holon::Record::def"`.

---

## Honest deltas

### Cascade rounds

**One round.** Added the variant → `cargo build --release -p wat` → 0 errors on first attempt. All probes + lib baseline green on first test run. The compiler reported no non-exhaustive match warnings — every site was identified from the known-site list in the DESIGN + exhaustive review of `grep -n "wat__holon__Record"`.

The `assoc` function required a structural decision: base early-returns before the holonic extraction to avoid needing an `Option<Arc<HolonAST>>` for `holon_form`. This is the cleanest shape (no flavor flag, no Option on a private binding) and leaves the holonic path byte-for-byte untouched.

### `RuntimeError::MalformedForm` for teaching errors

`TypeMismatch.expected` is `&'static str` — cannot carry a dynamic class name. `MalformedForm { reason: String }` is the honest carrier for per-instance diagnostic text. The teaching message is identical to the BRIEF's contract; the error variant differs from what was anticipated but is the substrate-honest choice.

### Files outside `src/` — three: `edn_shim.rs`, `closure_extract.rs`, and the probe

`edn_shim.rs` and `closure_extract.rs` had exhaustive matches over `Value` that needed the base arm. Both received or-pattern extensions — no behavioral change for base (same EDN rendering, same not-implemented closure-extract). `holon-rs` untouched. STOP-5 not triggered.

---

## Working tree on return

```
 M src/closure_extract.rs
 M src/edn_shim.rs
 M src/runtime.rs
?? tests/probe_arc237_sC2c_base_record.rs
```

holon-rs untouched. DO NOT commit (orchestrator commits).
