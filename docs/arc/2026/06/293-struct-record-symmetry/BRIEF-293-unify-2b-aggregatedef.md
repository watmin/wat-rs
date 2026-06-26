# BRIEF — arc 293 base-struct unification, strike 2b: `StructDef`+`RecordDef` → `AggregateDef{kind}`

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If the work exceeds these rooms or hits a STOP trigger,
STOP and report — do not improvise. This is a LARGE behavior-preserving cascade; **ride the compile cascade to
zero** (the fail-count is the progress meter) and STOP if it spreads past the map below.

Build/test: `cargo build --release -p wat`; `cargo test --release -p wat …`. **TRUST ONLY FORCED CLEAN BUILDS**
(`cargo clean -p wat && cargo build --release -p wat`) before claiming green — incremental builds + rust-analyzer
lag whipsaw this kind of change (stale `E0xxx` diagnostics that don't reflect the final source). Read the disk, not the cache.

## Decisions pinned (four-questioned + builder-confirmed — do NOT re-litigate)

- **D1 — `AggregateKind` is 3-way `{Struct, Record, HolonRecord}`.** `parent` is DERIVED from kind and DROPPED.
- **D2 — fields always-typed** (already true after strike 2a: `RecordDef.fields: Vec<(String, TypeExpr)>`).
- **`is_portable_type = kind != Struct`** (structs never cross the wire — R8/4b-i; records core+holon do).

## The work, in one paragraph

Merge `StructDef` + `RecordDef` into ONE **`AggregateDef`** carrying a `kind: AggregateKind`, and collapse the
two `TypeDef` variants `Struct(StructDef)` + `Record(RecordDef)` into one **`TypeDef::Aggregate(AggregateDef)`**.
The 3-way kind replaces (a) the variant tag and (b) the record `parent` string (core-vs-holon). `parse_defstruct`
emits `kind: Struct`; `parse_recordtype` maps its parent arg (`:wat::Record`→`Record`, `:wat::holon::Record`→
`HolonRecord`) to the kind and drops `parent`; every `register_builtin(TypeDef::Struct(…))` becomes
`Aggregate{kind: Struct}`. Every `TypeDef::Struct(sd)` / `TypeDef::Record(rd)` match site (≈50 + 38, **7 files**)
becomes `TypeDef::Aggregate(a)`: where struct and record did the **same** thing, the two arms **collapse to one**;
where they **differ**, branch on `a.kind`. **No value-construction change** — holon-vs-base `Value` is chosen by
the macro's `Record::of` vs `holon::Record::of` call, NOT by reading the def, so the runtime construction path is
untouched. Behavior-preserving: the SET-diff + green deterministic tests are the oracle (no RED probe).

## The shape to build (`src/types.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateKind { Struct, Record, HolonRecord }

impl AggregateKind {
    /// Holder root in the typesub lattice. None for Struct (structs register no parent edge today —
    /// StructDef had no `parent`). Records edge to their root.
    pub fn holder_root(&self) -> Option<&'static str> {
        match self {
            AggregateKind::Struct       => None,
            AggregateKind::Record       => Some(":wat::Record"),
            AggregateKind::HolonRecord  => Some(":wat::holon::Record"),
        }
    }
    /// The R8 wire wall: structs never cross; records (core + holon) do.
    pub fn is_portable(&self) -> bool { !matches!(self, AggregateKind::Struct) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateDef {
    pub name: String,
    pub type_params: Vec<String>,          // structs use; records leave empty
    pub fields: Vec<(String, TypeExpr)>,   // always-typed (D2)
    pub kind: AggregateKind,
    pub restrictions: Option<StructRestrictions>,  // Struct-only; None for records
}
// keep the field_names()/field_types() accessor methods (move them onto AggregateDef).
```
Then `TypeDef::Struct(StructDef)` + `TypeDef::Record(RecordDef)` → **`TypeDef::Aggregate(AggregateDef)`**.
**Annihilate `StructDef` and `RecordDef`.**

## Rooms — read in order (the cascade map; re-ground before editing)

1. **`src/types.rs:124-260`** — the def structs + the `TypeDef` enum + `TypeDef::name()`. Mint `AggregateKind` +
   `AggregateDef`; replace the two enum variants with `Aggregate`; `name()` collapses
   `Struct(s)`+`Record(r)` → `Aggregate(a) => &a.name`.
2. **`src/types.rs:420-445`** — the lattice registration (`if let TypeDef::Record(rec) … register_subtype(name,
   rec.parent)`). Becomes: `if let TypeDef::Aggregate(a) = &def { if let Some(root) = a.kind.holder_root() {
   register_subtype(a.name, root) } }`. (Struct's `None` → no edge, matching today.)
3. **`src/types/defstruct.rs:328-368`** — `parse_defstruct` → `Aggregate(AggregateDef { …, kind:
   AggregateKind::Struct, restrictions })`.
4. **`src/types.rs` `parse_recordtype` (~2160-2246)** — map the **parent arg** to the kind (`:wat::Record`→
   `Record`, `:wat::holon::Record`→`HolonRecord`, **any other parent → the existing "unknown parent" error**),
   drop the `parent` field, emit `Aggregate(AggregateDef { …, kind, type_params: vec![], restrictions: None })`.
5. **`src/types.rs` built-in registrations** (`grep -n "register_builtin(TypeDef::Struct"` — ~15 sites, e.g.
   `:514, :599, :881…`) → `register_builtin(TypeDef::Aggregate(AggregateDef { …, kind: Struct, restrictions: None }))`.
   ⚠ **The ROOT types** `:wat::Record` / `:wat::holon::Record` / `:wat::core::Struct` (`~:1409-1428`) — see STOP-1.
6. **`src/check.rs`** `is_portable_type` (`:13056` Record→true, `:13061` Struct→false) → ONE arm
   `Some(TypeDef::Aggregate(a)) => a.kind.is_portable()`. **Fix the stale `:12990` doc-comment** (it still says
   "Struct portable iff every field portable" — false since 4b-i). Plus the 293.3 surface arm: the
   `TypeDef::Struct(sd)` and `TypeDef::Record(rd)` branches (`grep struct_satisfies_surface`) collapse — both call
   `struct_satisfies_surface(&a.fields, …)` regardless of kind (records and structs satisfy surfaces identically now).
7. **`src/closure_extract.rs:2408`** — `WatAST::Keyword(r.parent.clone(), …)` → derive from kind:
   `a.kind.holder_root()` (the parent keyword the emitted form needs; Struct has none — handle that arm).
8. **`src/runtime.rs:1291-1292`** — `register_record_methods`' `parent != ":wat::Record" && parent != ":wat::holon::Record"`
   check → express via `a.kind` (it distinguishes a user record from the root types). If the intent is unclear
   when you read it in context, STOP-2.
9. **`grep -rn "TypeDef::Struct\|TypeDef::Record" src/`** — the remaining ≈88 match/construct sites across
   `check.rs`, `runtime.rs`, `edn_shim.rs`, `rete/kernel.rs`, `rete/matcher.rs`, `closure_extract.rs`, `types.rs`.
   For each: `TypeDef::Struct(sd)` and `TypeDef::Record(rd)` arms with **identical bodies → collapse to one
   `TypeDef::Aggregate(a)`**; arms that **differ → `TypeDef::Aggregate(a) => match a.kind { … }`** (preserve the
   exact per-kind behavior, including HolonRecord = the old Record behavior unless a site specifically special-cased holon).

## STOP triggers (halt + report; do NOT improvise)

1. **STOP-1 (the ROOT types).** `:wat::Record` / `:wat::holon::Record` / `:wat::core::Struct` are registered
   specially (`types.rs:~1409` NOTE: "registering `:wat::holon::Record` as a struct causes…"). Determine their
   correct `kind` (likely `Struct` — they are opaque umbrella holders, not user records). If their registration
   doesn't map cleanly to a kind, STOP and report exactly how they're registered + your proposed kind.
2. **STOP-2 (the `register_record_methods` parent check, `runtime.rs:1291`).** If you can't express the
   `parent != root` check via `a.kind` without changing behavior, STOP and report the intent you read.
3. **STOP-3 (a site that NEEDS `parent` beyond core/holon, or needs to distinguish a record's parent from its
   kind).** D1 says kind fully captures core-vs-holon. If a site reads `parent` for something `holder_root()`
   can't reconstruct, STOP — that would mean the parent isn't fully kind-derivable.
4. **STOP-4 (value construction).** You should NOT need to touch holon-vs-base `Value` construction
   (`Record::of`/`holon::Record::of`, `runtime.rs:~3897/13186/13247`) — it's macro-chosen. If the merge forces a
   change there, STOP and report (it means the kind leaked into value construction unexpectedly).
5. **STOP-5.** You are a LEAF. If the cascade spreads beyond the 7 files + `types/defstruct.rs` into unrelated
   subsystems, or is simply too large to land coherently, STOP and report the site list before mass-editing.

## Gate (the orchestrator re-runs every line against the disk — STRONG deterministic coverage, the floor is noise)

| what | command | expected |
|---|---|---|
| forced clean build | `cargo clean -p wat && cargo build --release -p wat` | clean (no `error[E…]`) |
| the wire wall holds (struct ↛ wire) | `cargo test --release -p wat --test nursery channel_of_all_edn_struct` (if present) + `probe_arc272_6a_capability_handoff --test probe_arc272_6c2_record_ipc_derisk` | green |
| records satisfy surfaces + structs do | `cargo test --release -p wat --test probe_arc293_record_surface --test probe_arc293_structural_surface --test probe_arc293_structtype_primitive` | 3 + 2 + 1 passed |
| record construct/accessor/EDN/holon/liskov | `cargo test --release -p wat --test test -- core_record_def` | 7 passed |
| struct construct/accessor | `cargo test --release -p wat --test test -- defstruct` (+ any struct deftests) | green |
| services (defservice uses struct State + record wire) | `cargo test --release -p wat --test test -- counter_on seeded admin_stop hibernate_resume` | green |
| no new regressions | `cargo test -p wat --no-fail-fast`, failing-test **SET** vs HEAD (`15157c3d` code, floor ≈ 201-204, **nondeterministic** arc-170 execve-leak NURSERY/holon-global class) | **∅ new** — weigh by SET; isolate any suspicious deterministic-named fail and baseline-check it |

## Report back
Full `git diff --stat`; verbatim gate output for each row; the failing-test SET (sorted); a note on EACH STOP
trigger (fired or not, with what you found — ESPECIALLY STOP-1 the root types); any TypeDef site where you had
to branch on kind vs collapse, and why. Do NOT commit — leave it uncommitted for the orchestrator's weigh.

Runtime: 90–150 min (large mechanical cascade). Trap-doors: (a) the root-type registrations (STOP-1); (b) sites
that branch struct-vs-record subtly (preserve per-kind behavior — don't over-collapse); (c) the cache/diagnostic
whipsaw (forced clean build only). The whole strike is behavior-preserving — the deterministic gate + SET-diff ∅
are your truth oracle; the nursery/holon leak floor is noise.
