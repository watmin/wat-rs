# BRIEF — 293.R2.2: ONE `register_aggregate_methods` for accessor codegen (the parity break dies)

**The work, in one paragraph.** After R2.1 there is ONE value repr (`Value::Aggregate`). But accessor codegen is
still split + broken: the `defrecord`/`defholon` wat macros (`wat/Record.wat`) emit per-field accessors as
`defn`s whose name is **string-concatenated** `"{fqdn-str}/{field}"` — for a generic record `:t::R<T>` that yields
`:t::R<T>/v` (the `<T>` lands mid-name where `defn`'s name parser can't strip it), so `:t::R/v` is never registered
(the catastrophic parity break). Meanwhile `register_struct_methods` mints struct accessors correctly in Rust
(generic-aware, bare key). **Fix: mint ALL field accessors in ONE Rust `register_aggregate_methods`** over
`AggregateDef` (every holder), generic-aware (bare `:T/field` key + carried `type_params`, exactly the struct path) —
and **remove the macro's accessor emission**. Plus the root cause the crawl found: **`parse_recordtype` stores the
mangled `:t::R<T>` as the type name** instead of calling `parse_declared_name` like every other decl parser — fix it
so the type registers at bare `:t::R` + `type_params`. The CTOR stays where it is (struct `/new` in
`register_struct_methods`; record/holon ctor in the macro — holon lowering lives in wat, do NOT move it). Result: the
`293.R2` parity probe goes GREEN (`(:r2::probe)` = 60).

## The one contract decision (pinned)
ONE `register_aggregate_methods(types, sym)` walks every `TypeDef::Aggregate(a)` and registers, per field, an
accessor `Function` — SHARED for all holders (the repr is one now):
- **key** = `format!("{}/{}", a.name, field_name)` with `a.name` BARE (the parse_recordtype fix guarantees this).
- **generic** = `type_params: a.type_params.clone()`, `param_types: vec![ parametric_decl_type(&a.name, &a.type_params) ]`
  (the helper at runtime.rs:895 — the struct path's exact pattern; this is what makes `:t::R/v` generic + bound).
- **body** = `(:wat::core::struct-field self <idx>)` — ONE field primitive for ALL holders (post-R2.1 `struct-field`
  reads any `Value::Aggregate(a).fields[idx]`; confirm `eval_struct_field` handles every holder, then use it
  uniformly — no more `struct-field` vs `Record/field-at` split in codegen).
- **index** = absolute position in the full field list (inherited ++ own); Struct has no inherited (reuse
  `collect_all_record_fields` for Record/Holon, runtime.rs:1268). `DuplicateDefine` guard for genuine collisions; a
  GRACEFUL skip is NOT needed once the macro no longer emits accessors (see STOP-2).

## Read in order (the rooms — grounded 2026-06-28, post-R2.1)
1. **`src/types.rs:2119-2131` + ~2261 (`parse_recordtype`)** — replace the raw `k.clone()` name with
   `parse_declared_name("recordtype", &name_kw, &decl_span)?` (returns `(bare_name, type_params)` — the exact call
   `parse_defstruct` @defstruct.rs:335 + parse_newtype/typealias/typeunion all use). Pass the extracted `type_params`
   into the `AggregateDef` (drop the hardcoded `vec![]` at ~2261). This makes the TYPE register at bare `:t::R`.
2. **`src/runtime.rs:925` (`register_struct_methods`)** — EXTRACT its accessor loop (≈984-1015, the `struct-field`
   body + `parametric_decl_type` + `type_params` + bare-key — the worked template) into the new
   `register_aggregate_methods`, generalized to match `TypeDef::Aggregate(a)` for ALL holders. The struct **ctor**
   loop (948-982, `/new`) STAYS in `register_struct_methods`.
3. **`src/runtime.rs:1316` (`register_record_methods`)** — it is vestigial (skips when the macro registered the ctor).
   Its inherited-field handling (`collect_all_record_fields`) moves into `register_aggregate_methods`; then **delete
   `register_record_methods`**.
4. **`wat/Record.wat`** — the `defrecord` macro (the accessor `map` block ≈125-162) + the `defholon::defrecord` macro
   (the parallel block): **REMOVE the accessor emission** (the `accessors` let + its splice). KEEP the `recordtype`
   decl + the ctor `(defn ~fqdn …)`. The Rust path now owns accessors. (The ctor's holon-form lowering is untouched.)
5. **`src/freeze/env.rs`** (+ `src/lib.rs:162` re-export) — call `register_struct_methods` (ctor only now) AND
   `register_aggregate_methods` (accessors, all holders). Swap the `register_record_methods` call → `register_aggregate_methods`.

## STOP triggers (halt + surface)
- **STOP-1 (`a.name` still mangled):** if after the `parse_recordtype` fix `a.name` is still `:t::R<T>` (the fix
  didn't take, or the name flows from elsewhere) — STOP and report; the accessor key depends on it being bare.
- **STOP-2 (the macro still emits accessors → DuplicateDefine):** the macro's accessor block MUST be removed in the
  same strike, else Rust + macro both register `:T/field` → `DuplicateDefine`. If removing it breaks the macro
  expansion (e.g. a dangling splice), STOP and report the exact macro shape.
- **STOP-3 (`struct-field` doesn't read every holder):** if `eval_struct_field` (runtime.rs:12053) rejects a
  Record/HolonRecord `Value::Aggregate` — STOP; either it should accept all (post-R2.1 it's one repr) or report why.

## The gate (orchestrator re-runs forced-clean)
- `cargo build --release -p wat` → clean.
- **The `293.R2` parity probe GREEN (un-ignore it):** `cargo nextest run --release -E 'test(aggregate_codegen_parity_generic_record_accessors)'` → PASS, `(:r2::probe)` = **60** (generic core-record + holon-record + struct accessors all resolve; verify it's 60, not a scramble).
- `grep -n 'fn register_record_methods' src/runtime.rs` → no hit.
- Whole workspace: `cargo nextest run --release` → floor 0, SET-diff ∅ vs HEAD (the un-ignored probe is +1 pass / −1
  skip → `4098 passed / 0 failed / 93 skipped`). Oracle suites green: `-E 'test(core_record_def) + test(holon) + test(defstruct) + test(kwargs) + binary(types)'`.

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally; let the cascade waterfall. Read every diff. Trust only forced-clean builds. STOP +
report if a STOP fires.
