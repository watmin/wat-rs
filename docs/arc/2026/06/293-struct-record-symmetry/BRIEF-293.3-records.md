# BRIEF — arc 293.3-records (the def-unification, strike 1): records carry typed fields → satisfy surfaces

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If the work exceeds these rooms or hits a STOP trigger,
STOP and report — do not improvise a workaround.

Build/test: `cargo build --release -p wat`, `cargo test --release -p wat …`. After editing any `wat/*.wat`,
**`touch tests/test.rs`** (wat-tests re-scan on `.rs` recompile). Trust forced clean builds
(`cargo clean -p wat && cargo build --release -p wat`) if results look stale.

## The work, in one paragraph

293.3-core made **structs** satisfy `defsurface`s (`StructDef.fields` is typed). Records *carry* their field
types in `RecordDef.field_types` — but `:wat::Record::def` and `:wat::holon::Record::def` emit the **string-
literal** `recordtype` form (`["name" …]`), leaving `field_types = None`, so a record can't satisfy a surface
even though it has the members. `recordtype` **already** parses the typed form `[name <- :type …]` →
`field_types = Some` (`types.rs:2161-2229`). This strike makes both record macros emit that typed form, and
adds a **Record arm** to `assignable` mirroring the Struct arm — so **core AND holon records satisfy surfaces**
by the same width-match as structs (the R2 headline). After this, `RecordDef` carries the same typed-field data
as `StructDef` — the precondition for the eventual `AggregateDef` merge (strike 2, not this strike).

## THE GATE = the committed RED probe goes GREEN

`tests/probe_arc293_record_surface.rs` (already committed, verified RED at HEAD):
- `core_record_structurally_satisfies_a_defsurface` — RED→**GREEN**
- `holon_record_structurally_satisfies_a_core_surface` — RED→**GREEN**
- `record_missing_a_surface_member_is_rejected` — **stays green** (guard: the surface is a real lower bound)

## Rooms — read in order (exact file:line; re-ground before editing)

1. **`wat/Record.wat:96-113`** — `:wat::Record::def`'s `recordtype` emission. Today it maps the field children
   to **name-strings** via a `:wat::holon::from-wat` round-trip, emitting `(:wat::core::recordtype ~fqdn
   :wat::Record [~@name-strs])`. **Replace the whole `[~@(let [...name-strs...] name-strs)]` block (lines
   97-113) with `[~@fields]`** — splice the user's ORIGINAL typed vector directly. This is exactly what the
   **ctor** does one line below at **`:114`** (`(:wat::core::defn ~fqdn [~@fields] -> ~fqdn …)`), so the splice
   is already proven to work. `recordtype`'s typed branch (`types.rs:2161`) parses `[name <- :type …]` and
   populates `field_types = Some`. Net: a DELETION (the name-strings holon round-trip dies) + `field_types` now populated.
2. **`wat/Record.wat:187-204`** — `:wat::holon::Record::def`'s `recordtype` emission. **Identical change:**
   replace the `[~@(let [...name-strs...] name-strs)]` block with `[~@fields]`. The two macros stay parallel.
3. **`src/check.rs:14237-14247`** — the `assignable` surface arm. Today, inside `if let TypeExpr::Path(ap) = &a`,
   it has ONE branch: `if let Some(TypeDef::Struct(sd)) = types.get(ap) { … struct_satisfies_surface(&sd.fields, …) }`.
   **Add a Record branch beside it** (see sketch). `struct_satisfies_surface` (`types/surface.rs:26`) is generic
   over `&[(String, TypeExpr)]` — reuse it verbatim with the record's `(name, type)` pairs.

## Implementation sketch (fill it; do not reinvent the shape)

```rust
// src/check.rs — inside `if let TypeExpr::Path(ap) = &a { … }`, AFTER the existing Struct branch
// (mirror it exactly; pure TypeEnv, no CheckEnv/SymbolTable):
if let Some(crate::types::TypeDef::Record(rd)) = types.get(ap) {
    // field_types = Some once Record::def emits the typed form. None (string-literal record) → cannot satisfy.
    if let Some(fts) = rd.field_types.clone() {
        let names = rd.field_names.clone();              // clone to release the `types` borrow
        let pairs: Vec<(String, crate::types::TypeExpr)> =
            names.into_iter().zip(fts).collect();
        return crate::types::surface::struct_satisfies_surface(
            &pairs,
            &surf_clone,
            |fty, mty| assignable(fty, mty, subst, types),
        );
    }
    // field_types == None → fall through to unify (no types to match → cannot structurally satisfy).
}
```

```clojure
;; wat/Record.wat — the recordtype emission, BOTH macros (base :96, holon :187):
;; BEFORE:  (:wat::core::recordtype ~fqdn :wat::Record [~@(:wat::core::let [...name-strs...] name-strs)])
;; AFTER:   (:wat::core::recordtype ~fqdn :wat::Record [~@fields])
;; (holon: parent :wat::holon::Record; otherwise identical)
```

## Decision pinned (do NOT re-litigate / do NOT exceed)

- **POPULATE `field_types`, do NOT restructure `RecordDef`.** This strike makes `field_types` always `Some`
  (via the typed emission) + adds the `assignable` arm. Do **NOT** merge `field_names`+`field_types` into a
  single `fields: Vec<(String, TypeExpr)>` — that shape-merge is the `AggregateDef` strike (strike 2), out of scope here.
- **The Record arm mirrors the Struct arm exactly** — pure `TypeEnv`, no `CheckEnv`/`SymbolTable`.
- **Both record macros change identically** (base + holon stay parallel). Do NOT touch struct surfaces (293.3-core, done).
- **No `Record::def`→`defrecord` rename here** (separate later strike — keep the `:wat::Record::def` head).

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if `[~@fields]` does not splice cleanly into `recordtype`** (a hygiene/quote error, or `recordtype`
   rejects the emitted form). The ctor at `:114` proves `[~@fields]` works, so this is unlikely — but if it
   fails, the fallback is to build the typed triples from the per-field name+type extraction the macro ALREADY
   does for accessors (`Record.wat:143-177` extracts `name-s` + `type-w`). Report which path you took; STOP if neither works.
2. **STOP if emitting the typed `recordtype` regresses existing record behavior** — `register_record_methods`,
   EDN round-trip, the ~75 `:wat::Record::def` call sites. `field_types` `None`→`Some` is **additive**
   (`register_record_methods` already handles the `Some` "direct user code" path). The SET-diff ∅ gate is the
   oracle; if a NON-probe record test regresses, STOP and report the set.
3. **STOP if the `assignable` Record arm needs `CheckEnv`/`SymbolTable`** — it must not (field_types lives in
   the `TypeDef`, pure `TypeEnv`, exactly like the Struct arm). If you reach for the symbol table, you've drifted.
4. **STOP if the holon macro's typed emission must diverge from the base** — they are parallel; the only
   difference is the parent keyword (`:wat::Record` vs `:wat::holon::Record`).
5. You are a LEAF. Do NOT spawn subagents. If the change exceeds these rooms, STOP and report.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| the record-surface probe goes green | `cargo test --release -p wat --test probe_arc293_record_surface` | **3 passed** (2 satisfy RED→GREEN; guard stays green) |
| struct surfaces still hold | `cargo test --release -p wat --test probe_arc293_structural_surface` | 2 passed (unchanged) |
| structtype parity intact | `cargo test --release -p wat --test probe_arc293_structtype_primitive` | 1 passed |
| records still work (ctor/accessor/EDN) | `cargo test --release -p wat --test test` (the `Record::def`/record deftests) + `--test probe_arc234_stone2a_record_primitives` | green |
| no new workspace regressions | `cargo test -p wat --no-fail-fast`, failing-test **SET** vs HEAD (`560535a5` code) | **∅** new (floor ≈ 201; weigh by SET, never absolute count) |

Runtime: 30–60 min. Trap-doors: (a) the `[~@fields]` splice into `recordtype` (proven by the ctor, but verify
`field_types` actually becomes `Some` — a quick check: the probe going green IS that proof); (b) any record
test that asserted the OLD string-literal `recordtype` emission shape (it shouldn't — that's internal). Report
the full `git diff --stat`, the verbatim gate output, and any honest deltas; do NOT commit.
