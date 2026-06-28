# BRIEF — 293.R2a: ONE `register_aggregate_methods` for field-accessor codegen (holder = the only variance)

**The work, in one paragraph.** Field-accessor synthesis is two drifted Rust functions: `register_struct_methods`
(`runtime.rs:924`, `holder == Struct`) carries `type_params` + uses the **bare** name for the accessor key;
`register_record_methods` (`runtime.rs:1315`, `holder != Struct`) hardcodes `type_params: vec![]` + builds the
key from `entry.name` which **carries the `<T>`** — so a generic record/holon-record's accessor lands at the
mangled key `:R<T>/v` and `:R/v` is never registered (the catastrophic parity break). **Merge the accessor
synthesis into ONE `register_aggregate_methods` that walks every `TypeDef::Aggregate`, generic-aware, bare key,
with the holder selecting ONLY the per-field accessor primitive.** Leave the **constructors** exactly as they are
(struct `/new` in `register_struct_methods`; record/holon ctor in the `defrecord` macro) — ctor unification is the
named follow-on R2b. This is behavior-preserving **except** generic records/holon-records now get their accessors.

## The one contract decision (pinned)
`register_aggregate_methods(types: &TypeEnv, sym: &mut SymbolTable) -> Result<(), RuntimeError>` iterates
`types.iter()`, matches **every** `TypeDef::Aggregate(a)` (all three holders), and for each field registers an
accessor that is **SHARED** in everything except the holder-policy axis:

- **SHARED (written once):**
  - **key** = `format!("{}/{}", a.name, field_name)` — `a.name` is the BARE name (no `<T>`; confirm via the struct
    path which already does this). This is the fix: records use the bare name, never `entry.name`-with-`<T>`.
  - **generic-aware** = `param_type = parametric_decl_type(&a.name, &a.type_params)` (the helper at `runtime.rs:895`);
    `Function.type_params = a.type_params.clone()`. (Struct does this; records now inherit it.)
  - **shape** = `Function { name, params: vec!["self".into()], param_types: vec![<the aggregate type>], ret_type:
    field_type.clone(), type_params, body, .. }`; the `DuplicateDefine` collision guard.
  - **index** = absolute position in the FULL field list (inherited ++ own). For a Struct there are no inherited
    fields → position in `a.fields`. For Record/HolonRecord → `inherited_count + own_index` (the existing
    `collect_all_record_fields` path, `runtime.rs:1268`/`1355`).
- **HOLDER-POLICY (the only variance — a `match a.holder`):**
  - **accessor body primitive:** `Holder::Struct` → `(:wat::core::struct-field self <idx>)`;
    `Holder::Record | Holder::HolonRecord` → `(:wat::Record/field-at self <idx>)`.
  - **inherited fields:** `Struct` → none; `Record | HolonRecord` → collect from the parent chain when the parent
    is a non-root extensible record (the `ROOT_PARENTS` skip + `collect_all_record_fields`, runtime.rs:1350-1359).

The body's single param is named **`self`** for all holders (struct uses `self`, record uses `v` today — unify to
`self`; it is an internal name the body references, so unifying is safe and is part of "one toolkit").

## Read in order (the rooms — grounded 2026-06-28)
1. **`runtime.rs:924-1042` (`register_struct_methods`)** — the **ctor** loop (948-982) STAYS here (struct `/new`,
   unchanged). EXTRACT the **accessor** loop (≈984-1015) — its shape (`struct-field` body, `parametric_decl_type`,
   `type_params` carried, bare-name key, DuplicateDefine) is the template for the shared accessor.
2. **`runtime.rs:1315-1465` (`register_record_methods`)** — its ENTIRE job is accessors (the ctor comes from the
   macro). The `RecordEntry` snapshot + `collect_all_record_fields` inherited-field handling (1334-1359) + the
   accessor loop (1434-1464) FOLD into `register_aggregate_methods` as the Record/Holon branch. **Fix the two
   drifts:** key uses the BARE `a.name` (not `entry.name`-with-`<T>`); `type_params: a.type_params.clone()` (not
   `vec![]`). After folding, **delete `register_record_methods`**.
3. **`runtime.rs:895` (`parametric_decl_type`)** + **`runtime.rs:1268` (`collect_all_record_fields`)** — reuse
   verbatim; do not reimplement.
4. **`src/freeze/env.rs:27-28` + the call sites** — today calls `register_struct_methods` AND
   `register_record_methods`. After: call `register_struct_methods` (ctor only now) AND
   `register_aggregate_methods` (accessors for all). Order: accessors can register after ctors; keep the existing
   relative order, just swap `register_record_methods` → `register_aggregate_methods`. Update `src/lib.rs:162`
   re-export if needed.
5. **`src/types.rs` `Holder` / `AggregateDef`** — `a.name` (bare), `a.type_params`, `a.fields`, `a.holder`,
   `a.parent` are the fields you read. Confirm `a.name` is bare for a generic decl (the struct path's correctness
   depends on it; if records stored a `<T>`-bearing name somewhere, that is the mangling — fix at the source).

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (`a.name` carries `<T>`):** if the generic record's `AggregateDef.name` itself is `":r2::CR<T>"`
  (not bare `":r2::CR"`), the mangling is at the *parser/registration*, not the accessor loop — STOP and report
  where the name is set, do not paper over it by stripping `<T>` in the accessor.
- **STOP-2 (struct-field vs field-at index mismatch):** if the struct `struct-field` primitive and the record
  `Record/field-at` primitive index differently (e.g. one 0-based, one 1-based, or holon offset) — STOP and
  report; the shared "absolute position" must match each primitive's contract.
- **STOP-3 (double registration):** if folding the record accessors causes a `DuplicateDefine` because the
  `defrecord` macro ALSO emits accessors — STOP and report (it means accessors are double-sourced; that's a
  macro-thinning question for R2b, surface it).

## The gate (the disconfirming probe, committed RED + `#[ignore]`'d)
`tests/types/probe_arc293_r2_aggregate_codegen_parity.{rs,wat}` — a generic core-record, generic holon-record,
generic struct each expose `/v`; a holon record passes where `:wat::Record` is wanted. Verified RED at HEAD
(`:r2::CR/v` + `:r2::HR/v` unresolved). **UN-IGNORE it; it goes GREEN** → `(:r2::probe)` = 60.

## EXPECTATIONS — see `EXPECTATIONS-293.R2a.md`.

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally (`cargo build --release -p wat`; let the cascade waterfall). Read every diff. Trust
only forced-clean builds for green. Self-verify the EXPECTATIONS. STOP + report if a STOP fires.
