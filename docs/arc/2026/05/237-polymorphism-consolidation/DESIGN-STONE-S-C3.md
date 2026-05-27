# Stone S-C.3 — the macro split: base `:wat::Record::def` / holonic `:wat::holon::Record::def`

**Status:** sub-DESIGN. Depends: S-C.2c ✓ (base variant), S-C.2d ✓, arc 238 ✓ (`=` on records).
Closes the records flavor thread (with S-D migration). Parent: `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`.

## What this does

Splits the one record-defining macro into two flavors, **flipping the unmarked name to the cheap
common case** (the user's locked model):
- **`:wat::Record::def`** → **BASE** record (struct only; the common, cheap record).
- **`:wat::holon::Record::def`** → **HOLONIC** (struct + holon; opt-in for holon-ops).

This is what finally **constructs** the base variant S-C.2c minted — base records become real at the
wat surface. The static type distinction *is* the constructor return type / recordtype parent.

## Grounded mechanism (all pieces exist)

- **Constructors** (`src/runtime.rs`): current `:wat::Record::of` is 3-arg holonic
  (`eval_record_of`, 16540: class + struct + holon_form → `wat__holon__Record`). Split:
  - **Rename** current → **`:wat::holon::Record::of`** (3-arg, holonic; unchanged body).
  - **Mint** **`:wat::Record::of`** (2-arg: class keyword + struct Vec → `Value::wat__Record`). A
    stripped `eval_record_of` minus the holon_form arg/branch. ~20 lines.
- **Macros** (`wat/Record.wat`): current `:wat::Record::def` builds holonic (Bind/Bundle holon_form
  + calls `:wat::Record::of`). Split:
  - **Rename** current macro → **`:wat::holon::Record::def`**: emit `(recordtype :Name :wat::holon::Record [names])`
    + constructor calling **`:wat::holon::Record::of`** (the renamed 3-arg) + accessors. (Body = current,
    minus the parent + constructor-verb change.)
  - **Mint** **`:wat::Record::def`** (BASE): emit `(recordtype :Name :wat::Record [names])` + constructor
    calling **`:wat::Record::of`** (2-arg — NO holon_form Bind/Bundle block; just class + struct syms) +
    accessors + return type `-> :wat::Record`. **Simpler than holonic** (drops the entire holon_form
    construction). Accessors + predicate identical shape (field-at is variant-agnostic; is-X? auto-minted).
- **Flavor hierarchy** (`src/types.rs`, ALREADY built): `:wat::holon::Record` is a registered type
  (1397) with seeded edge `:wat::holon::Record is-a :wat::Record` (1404). `recordtype`'s arg[1] =
  parent. So a base-defined `:my::Pt` → `<: :wat::Record`; a holonic-defined `:my::HPt` →
  `<: :wat::holon::Record <: :wat::Record`. Liskov falls out of existing `is_subtype` (S-A1): a func
  `[v <- :wat::holon::Record]` REJECTS a base-defined record; `[v <- :wat::Record]` accepts BOTH.

## The cascade (S-D — coordinated, atomic green commit)

Flipping `:wat::Record::def` holonic→base breaks the ~23 caller files that reference it — but only
the ones whose tests actually exercise **holon-ops** on the instance. Per `cargo test`
(substrate-as-teacher), migrate per this rule:

> **Stays BASE** (`:wat::Record::def`, no edit) unless the record instance is fed to holon-ops
> (`to-holon` / `:wat::holon::` extraction / holon auto-dispatch) → then migrate to
> **`:wat::holon::Record::def`**. Default base; holonic by demonstrated need.

Most of the 23 silently-and-correctly become base and keep passing (field-access / predicate / `=` /
`same-data?` / `assoc` / `record->map` all work on base). Expected migrations: the holon-using probes
(`probe_arc234_stone5_holon_auto_dispatch`, and any to-holon/`:wat::holon::`-on-record sites the
cascade surfaces). **The cascade fail-count is the migration worklist; ride it to green.** S-C.3
(flip mechanism) + S-D (migration) commit ATOMICALLY when the tree is green (no broken commit).

## Coverage mandate (per `feedback_logic_coverage_mandate` — prove the ground)

The FM-2-bis probe `tests/probe_arc237_sC3_macro_split.rs` must cover the FULL logic surface:

**Base (`:wat::Record::def :my::Pt [x y]`):**
1. construct `(:my::Pt 1 2)` → a base record (type `:wat::Record`; NOT `:wat::holon::Record`).
2. field accessor `(:my::Pt/x p)` / keyword `(:x p)` → field value.
3. predicate `(:my::is-Pt? p)` → true; on a different class → false.
4. `=` base-vs-base same data → true; diff → false.
5. `same-data?` base-vs-base → true.
6. `assoc` → new base record (struct rebuilt; still base).
7. **`to-holon` on base → ERROR** (the teaching error; base has no holon flavor).

**Holonic (`:wat::holon::Record::def :my::HPt [x y]`):**
8. construct → holonic record; field access + predicate + `=` + `same-data?` all work.
9. **`to-holon` on holonic → OK** (holon-ops work — the flavor's whole point).

**The Liskov type-distinction (LOAD-BEARING — the static proof):**
10. func `[v <- :wat::Record]` accepts a base-defined record (Ok check).
11. func `[v <- :wat::Record]` accepts a holonic-defined record (Ok — holonic <: base).
12. func `[v <- :wat::holon::Record]` accepts a holonic-defined record (Ok).
13. **func `[v <- :wat::holon::Record]` REJECTS a base-defined record (check ERROR)** — the
    flavor distinction enforced at compile time.

**Cross-flavor (depends both macros):**
14. `same-data?` base-`:my::Pt`[0,0] vs holonic-`:my::HPt`[0,0] (same field names) → true (type-blind).
15. `=` base vs holonic → false (different type/flavor).

## Trap-doors (REJECTION)
1. `:wat::Record::def` must build BASE (not holonic). The UNMARKED name = the cheap common case.
2. Base macro must NOT emit holon_form / Bind / Bundle — base has no holon flavor.
3. recordtype parent must be correct per flavor (`:wat::Record` base / `:wat::holon::Record` holonic) —
   this IS the Liskov mechanism; getting it wrong breaks contract 13.
4. Migration: do NOT migrate a caller to holonic unless it uses holon-ops (default base).
5. Atomic green commit only — no broken intermediate on disk.
6. holon-rs untouched. Non-obvious error → STOP + surface.

## Slicing
One coordinated cascade flight: flip mechanism (constructors + macros + recordtype parents) → ride
cargo test to migrate holon-op callers → comprehensive probe green → atomic commit. (If the cascade
proves larger than predicted, S-D splits off as a second sweep against the dirty tree, atomic commit.)

## Calibration
Meatier than recent stones: 2 constructor changes (rename + mint base) + 2 macro changes (rename +
mint base) + recordtype-parent wiring + a migration cascade + comprehensive coverage probe. **Target
band: 60–100 min Mode A; 120 STOP-3; 150 STOP-4.** (The macro authorship + cascade is the cost.)
