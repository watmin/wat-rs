# 293.R2 — the aggregate codegen annihilation: ONE toolkit, holder is the only variance

> **Status: STRIKE — lair studied 2026-06-28. This is R2's (`FRANGE UT UNUM FIAT`) unfinished fulfillment.**
> unify-2b merged the **data** (`StructDef`+`RecordDef` → `AggregateDef{holder}`). The **behavior** was never
> merged: ctor + accessor synthesis is still TWO Rust functions split by `holder == Struct` / `holder != Struct`,
> and they drifted on axes the holder does not own. This strike makes the codegen ONE thing.

## The bug (grounded 2026-06-28, the parity matrix)

| dimension | struct | core-record | holon-record | the lie |
|---|---|---|---|---|
| field accessor — mono | ✅ | ✅ | ✅ | — |
| field accessor — **generic `<T>`** | ✅ | ❌ `:T/v` unresolved | ❌ `:T/v` unresolved | record codegen drops `<T>` |
| construct via **bare `:T`** | ❌ needs `/new` | ✅ | ✅ | struct codegen never got the bare ctor |

Two breaks, opposite directions, **one root**: `register_struct_methods` (`runtime.rs:924`, `holder == Struct`)
and `register_record_methods` (`runtime.rs:1315`, `holder != Struct`) are **one job written twice**, each given
half the unification — struct got generic-decl handling (`parametric_decl_type` + carries `type_params`) but not
the bare ctor; record got the bare ctor (from the wat macro) but not generic-decl handling (`entry.name` carries
`<T>` → the accessor registers at the mangled key `:T<G>/v`, never `:T/v`; and `type_params: vec![]`).

## The contract decision (pinned — the builder, 2026-06-28)

> *"make it such that there's one toolkit for the holders — the only variance is the policy enforcement.
> structs cannot cross comms, records must edn-repr, holon-record may be passed where a caller wants a
> core-record."*

**ONE `register_aggregate_methods` mints ctor + per-field accessors for every `TypeDef::Aggregate`.** The
`holder` is a **narrow policy branch**, nothing else. Everything that is NOT holder-policy — param names, field
types, generic-decl handling (`parametric_decl_type`, carried `type_params`), the bare accessor key — is
**shared, written once.**

### The holder is the ONLY variance — and exactly these axes

| policy axis | Struct | Record | HolonRecord | enforced where |
|---|---|---|---|---|
| **value ctor primitive** | `:wat::core::struct-new` | `:wat::Record::of` (2-arg) | `:wat::holon::Record::of` (3-arg) | the synthesized ctor body |
| **field accessor primitive** | `:wat::core::struct-field` | `:wat::Record/field-at` | `:wat::Record/field-at` | the synthesized accessor body |
| **crosses comms?** | ❌ `is_portable=false` | ✅ | ✅ | `is_portable_type` (`check.rs:13313`) — KEEP, holder-keyed |
| **assignable where core wanted?** | only where Struct wanted | itself | ✅ `holon <: core` | the lattice edge (`register_subtype`) — KEEP, PASSES today |

The bottom two rows are **already correct and already holder-keyed** (verified live: `is_portable_type` gates
`send'`/`recv'`; `holon ⊂ core` passes — a holon record is accepted where `:wat::Record` is wanted). The merge
must **preserve** them, not rebuild them. The top two rows are the only thing the codegen branches on.

## What is SHARED (written once — the toolkit)
- Constructor at the type's keyword path; one param per field, declaration order; **generic-aware** for ALL
  holders (`parametric_decl_type(name, type_params)` for the param/ret types; `type_params` carried into the
  scheme). Body = `(<value-ctor-primitive> <type-kw> ~params)`.
- Per-field accessor at the **bare** `:T/<field>` key (the `<G>` lives in `type_params`, never the key —
  `register_struct_methods`' existing behavior, now the law for records too); body = `(<accessor-primitive> self idx)`.
- `DuplicateDefine` collision guard; inherited-field handling for extensible record parents (the
  `collect_all_record_fields` path — Record/HolonRecord only; Struct has no parent chain).

## Decomposition (sub-strikes — depth-first)

- **293.R2a — merge the ACCESSOR codegen (THIS strike — the catastrophic break).** One
  `register_aggregate_methods` mints **per-field accessors for EVERY `TypeDef::Aggregate`**, generic-aware
  (`parametric_decl_type` + carried `type_params`), bare key, holder picking ONLY the accessor primitive
  (`struct-field` vs `Record/field-at`). Extract the accessor loop out of `register_struct_methods`; replace
  `register_record_methods`'s accessor loop with the shared one (its inherited-field handling moves in too).
  `register_struct_methods` keeps **only** the struct ctor (`/new`, unchanged); the record/holon ctor keeps
  coming from the `defrecord` macro. **No macro change, no construction-form change, no `.wat` cascade.**
  Behavior-preserving EXCEPT generic records/holon-records now get their accessors. Gate = the parity matrix
  RED probe GREEN + the three policies green + SET-diff ∅. *(Why accessor-first: the record ctor lives in a wat
  macro whose full emissions need their own crawl — bundling that into the break-fix would put macro
  archaeology on a LEAF. Named, not deferred-in-costume: R2b.)*
- **293.R2b — unify the CONSTRUCTOR codegen.** Fold the struct ctor (from `register_struct_methods`) and the
  record/holon ctor (from the `defrecord` macro) into `register_aggregate_methods`, holder picking the value
  ctor primitive (`struct-new` / `Record::of` / holon `Record::of`). The `defrecord` macro thins to a
  `recordtype` emitter (struct already works this way via `structtype`). Construction **form** unchanged here
  (struct still `/new`, record still bare) — only the source moves. Requires the macro-emission crawl.
- **293.R2c — construction-form parity (the cascade).** Drop `:T/new` for structs → bare `:T` for all three
  (the decided "unify on `:T`", NOTE-base-struct-horizon). User-visible: ~8 `.wat` (`Launched/new` →
  `Launched`, …) + a handful of `.rs` fixtures via fix-wat + hand-sub. Separate strike (it turns the tree red
  across fixtures).
- **293.R2d — (optional) consolidate the scattered check-side `a.holder == Struct` branches** (`check.rs`
  12178/12656/12930) behind named predicates, so holder-policy lives in one vocabulary. Polish, not blocking.

## Out of scope (named, not deferred-in-costume)
- **Layer-2 repr collapse** (the 3 `Value` variants → 1 + tag) — the explicit optional horizon
  (`NOTE-base-struct-horizon.md`); high blast (serialization/wire-gate/every exhaustive match). NOT this strike.
- **`register_enum_methods` / `register_newtype_methods`** — siblings with the same shape, but enums/newtypes
  are not the three holders; leave them (a later "all codegen is one walk" could fold them, not now).
- **`/from-map`** — uniformly absent for all three (the blocked 291 driver); lands with 291's resume, not here.

## The gate (R2a)
`tests/types/probe_arc293_r2_aggregate_codegen_parity.{rs,wat}` — the parity matrix as one program: a generic
core-record and a generic holon-record each expose their field accessor (`:R/v`, `:H/v` resolve and return the
field); a struct accessor stays green; the three policies are each asserted (struct rejected at a comms verb;
holon accepted where `:wat::Record` wanted). RED at HEAD (the generic record/holon accessors are unresolved).

## Pairs
`REALIZATIONS.md` R2 *FRANGE UT UNUM FIAT* (open fulfillment — this closes it) · `NOTE-base-struct-horizon.md`
(Layer 1 vs 2; the `:T/new`→`:T` decision) · `runtime.rs:924`/`:1315` (the two functions) ·
`check.rs:13313` (`is_portable_type` — the comms wall to preserve) · `feedback_uniform_operation_or_decomplect_is_catastrophic`.
