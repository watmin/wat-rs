# SCORE — Stone S-C.3 — macro split + BASE/holonic flavor bifurcation

**Status:** COMPLETE. All targets met. Tree dirty (no commit — orchestrator commits).

## Scorecard

- [x] `:wat::Record::def` minted as BASE macro — emits `(recordtype ~fqdn :wat::Record [...])`, constructor calls 2-arg `:wat::Record::of`, NO holon_form/Bind/Bundle block, return type `-> :wat::Record`.
- [x] `:wat::holon::Record::def` minted as HOLONIC macro — emits `(recordtype ~fqdn :wat::holon::Record [...])`, constructor calls 3-arg `:wat::holon::Record::of`, includes full Bind/Bundle holon_form block, return type `-> :wat::holon::Record`.
- [x] `:wat::Record::of` (2-arg BASE) — new runtime fn + checker handler; produces `Value::wat__Record`.
- [x] `:wat::holon::Record::of` (3-arg HOLONIC) — renamed from old `:wat::Record::of`; body unchanged; produces `Value::wat__holon__Record`.
- [x] `is_holon_or_vector` extended to accept `:wat::holon::Record` — `cosine`/`dot` accept holonic records.
- [x] `is_holon_or_record` extended to accept `:wat::holon::Record` — `Bind`/`Bundle` accept holonic records.
- [x] `extract-classifier` dispatch arm extended to return `:String` (not `Option<String>`) for `:wat::holon::Record` args.
- [x] `is_atomizable` already updated (Stone S-C.3 prep) — `:wat::holon::Record` accepted for `to-holon`.
- [x] `infer_comparison` already updated (Stone S-C.3 prep) — `=` accepts base vs holonic via subtype fallback.
- [x] CASCADE: 5 probe files migrated per rule (stays BASE unless holon-op use confirmed):
  - `probe_arc234_stone1_wat_record_variant.rs` — Rust-level construction → `Value::wat__holon__Record`
  - `probe_arc234_stone15_namespace_promotion.rs` — Rust-level construction → `Value::wat__holon__Record`
  - `probe_arc234_stone2a_record_primitives.rs` — wat source uses `to-holon`; migrated to `:wat::holon::Record::of`/`def`
  - `probe_arc234_stone5_holon_auto_dispatch.rs` — uses all 5 holon verbs; migrated to `:wat::holon::Record::def`
  - `probe_diagnostic_typed_entities_reflection.rs` — probes 3/5/6 use `to-holon`; migrated those three
- [x] `probe_arc237_sC3_macro_split.rs` — 18/18; fixed `:wat::core::true` → `true` (keyword vs bool literal)
- [x] Full regression: 0 FAILED.

## Test result lines (exact)

```
cargo build --release -p wat 2>&1 | grep "^error"
  (no output — 0 errors)

cargo test --release --test probe_arc237_sC3_macro_split 2>&1 | grep "test result"
  test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

cargo test --release --lib -p wat 2>&1 | grep "test result"
  test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s

cargo test --release --test probe_arc238_eq_completeness 2>&1 | grep "test result"
  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --test probe_arc237_sC2d_same_data 2>&1 | grep "test result"
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --test probe_arc227_stone2_defrecord 2>&1 | grep "test result"
  test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

cargo test --release --no-fail-fast 2>&1 | grep -c "FAILED"
  0
```

## What was changed

### 1. `wat/Record.wat` — complete rewrite (macro split)

**BASE macro** (`:wat::Record::def`):
```
(define-syntax :wat::Record::def
  ...
  (recordtype ~fqdn :wat::Record [field-names...])
  (define (:ns::Name field...) -> :wat::Record
    (:wat::Record::of :ns::Name [field...]))
  ... accessors ...
)
```
No `holon-form` binding. No `Bind`/`Bundle` block. Constructor calls 2-arg `:wat::Record::of`.

**HOLONIC macro** (`:wat::holon::Record::def`):
```
(define-syntax :wat::holon::Record::def
  ...
  (recordtype ~fqdn :wat::holon::Record [field-names...])
  (define (:ns::Name field...) -> :wat::holon::Record
    (:wat::holon::Record::of :ns::Name [field...] (Bind (Atom class) (Bundle field-binds...))))
  ... accessors ...
)
```
Identical accessor codegen. Parent is `:wat::holon::Record`. Constructor calls 3-arg `:wat::holon::Record::of`.

### 2. `src/runtime.rs` — constructor split

Dispatch arms:
```rust
":wat::Record::of"        => eval_record_of(args, list_span, env, sym),
":wat::holon::Record::of" => eval_holon_record_of(args, list_span, env, sym),
```

`eval_record_of` — 2-arg BASE: extracts `class_fqdn` (keyword arg) + `struct_form` (vec); returns `Value::wat__Record { class_fqdn, struct_form }`.

`eval_holon_record_of` — 3-arg HOLONIC: was original `eval_record_of` body; extracts class + struct + HolonAST; returns `Value::wat__holon__Record { class_fqdn, struct_form, holon_form }`.

### 3. `src/check.rs` — checker extensions

- `infer_record_of` — arity changed 3→2; return type changed `:wat::holon::Record` → `:wat::Record`.
- `infer_holon_record_of` — new fn; 3-arg holonic; returns `:wat::holon::Record`.
- Dispatch arm added for `:wat::holon::Record::of` (calls `infer_holon_record_of`).
- `is_atomizable` — added `:wat::holon::Record` (base record is non-atomizable by design; holonic admits `to-holon`).
- `infer_comparison` — subtype fallback for `=` across `:wat::Record` / `:wat::holon::Record`.
- `is_holon_or_vector` — added `:wat::holon::Record` (for `cosine`/`dot`).
- `is_holon_or_record` — added `:wat::holon::Record` (for `Bind`/`Bundle`).
- `extract-classifier` dispatch arm — also returns `:String` (not `Option<String>`) for `:wat::holon::Record` args.

### 4. Migration cascade (5 test files)

Migration rule applied consistently: stays BASE unless record instance is fed to holon-ops (`to-holon`/`cosine`/`Bind`/`Bundle`/`extract-classifier`). `extract-classifier` alone does NOT require holonic (base records carry `class_fqdn`).

- `probe_arc234_stone1_wat_record_variant.rs` — Rust direct construction; `make_record` → `Value::wat__holon__Record`.
- `probe_arc234_stone15_namespace_promotion.rs` — same Rust pattern; same fix.
- `probe_arc234_stone2a_record_primitives.rs` — wat source uses `to-holon`; `:wat::Record::of` → `:wat::holon::Record::of`; return types updated.
- `probe_arc234_stone5_holon_auto_dispatch.rs` — all 5 holon verbs; `:wat::Record::def` → `:wat::holon::Record::def`.
- `probe_diagnostic_typed_entities_reflection.rs` — probes 3/5/6 call `to-holon`; those three migrated to `:wat::holon::Record::def`.

Probes NOT migrated (BASE stays correct):
- `probe_diagnostic_defprotocol_dispatch.rs` — only `extract-classifier`; BASE sufficient; 3/3 green.
- `probe_diagnostic_polymorphic_type.rs` — no holon-ops; BASE sufficient; 8/8 green.

## Probe fix: `:wat::core::true` → `true`

`probe_arc237_sC3_macro_split.rs` had `WANTS_BASE` and `WANTS_HOLON` constants using `:wat::core::true` as the function body of a `-> :wat::core::bool` function. `:wat::core::true` is a keyword literal (type `:wat::core::keyword`), not a boolean (type `:wat::core::bool`). Fixed to `true` (unquoted boolean literal). This was a probe authoring error, not a substrate defect.

## Honest deltas

- `wat/Record.wat`: complete rewrite; +~40 lines net (HOLONIC macro is near-copy of old; BASE macro is smaller).
- `src/runtime.rs`: +~35 lines (new 2-arg `eval_record_of`; old body renamed to `eval_holon_record_of`; dispatch arm +1 line).
- `src/check.rs`: +~60 lines net (new `infer_holon_record_of` ~30 lines; `infer_record_of` -5 lines; 4 predicate extensions ~8 lines; `extract-classifier` arm +3 lines).
- 5 test files migrated (Rust construction variant names; wat source macro/constructor names).
- New: `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C3.md` (this file).

## git status --short (relevant files)

```
 M src/check.rs
 M src/runtime.rs
 M wat/Record.wat
 M tests/probe_arc234_stone1_wat_record_variant.rs
 M tests/probe_arc234_stone15_namespace_promotion.rs
 M tests/probe_arc234_stone2a_record_primitives.rs
 M tests/probe_arc234_stone5_holon_auto_dispatch.rs
 M tests/probe_diagnostic_typed_entities_reflection.rs
?? tests/probe_arc237_sC3_macro_split.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C3.md
```
