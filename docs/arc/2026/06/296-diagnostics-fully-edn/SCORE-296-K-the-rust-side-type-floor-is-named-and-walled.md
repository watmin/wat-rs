# SCORE — 296 K: the Rust-side type floor is named and walled

No commit. Floor and clippy left to the orchestrator. Lands on H-2/H-2b/H-2c/H-3/J; those stones were not reverted.

J purged a class that admitted nothing. K's job was the harder sibling: when something **does** survive, its survival is argued and structurally enforced, so "cannot be wat" and "has not been moved yet" stop looking alike. The three movable aliases left Rust. The floor that remains is named, and the wall asks an oracle (aggregates) or prints a reason (aliases).

Sibling lint file `tests/lint/no_hand_written_type_floor.rs`, not J's `no_hand_written_enumdef.rs`: J admits NOTHING; K admits a floor. Folding them would make a red ambiguous between "an enum that should have moved" and "an aggregate that is a root".

---

## Row 1 — the alias macro exists

`grep -c 'pub fn wat_alias_register_from' crates/wat-source-derive/src/lib.rs` = **1**.

Third sibling of `wat_record_from!` / `wat_enum_register_from!`. Reads
`(:wat::core::typealias :ns::Name <type-form>)` (optional `:- [T …]` via the crate's one `binder_type_params`). The type form is source text parsed by `parse_type_expr_from_source` — no second parser.

## Row 2 — the movable aliases are IN WAT

Three `(:wat::core::typealias …)` declarations:

```
wat/holon.wat:293  :wat::holon::BundleResult  (:wat::core::Result :- [:wat::holon::HolonAST :wat::holon::CapacityExceeded])
wat/holon.wat:301  :wat::holon::Holons        (:wat::core::Vector :- [:wat::holon::HolonAST])
wat/core.wat:2137  :wat::core::Bytes          (:wat::core::Vector :- [:wat::core::u8])
```

Heads are the TypeExpr paths (`:wat::core::Result` / `:wat::core::Vector`), not the comment shorthand `Result`/`Vec`. `target/release/wat --check tests/types/probe_arc296_h2__variant.wat` loaded the stdlib **green** — `Existing::Equivalent` held (STOP-4).

The EXPECTATIONS grep also hits the transcribed comments that name the same aliases (6 lines). The three declarations are the ones that matter.

## Row 3 — their Rust literals are gone

`grep -c 'register_builtin(TypeDef::Alias' src/types.rs` = **1** — only `:wat::core::nil` (STOP-2).

The three moves are now:

```
wat_alias_register_from!(env, "wat/holon.wat", ":wat::holon::BundleResult");
wat_alias_register_from!(env, "wat/holon.wat", ":wat::holon::Holons");
wat_alias_register_from!(env, "wat/core.wat",  ":wat::core::Bytes");
```

## Row 4 — aggregate literals left are ONLY roots

Five `register_builtin(TypeDef::Aggregate` sites. Three carry a string-literal `name: "…"`; two are generated (STOP-3, no string-literal name):

| line | name | from_root_keyword | why it is here |
|---|---|---|---|
| 933 | `:wat::core::Struct` | `Some(Struct)` | category root |
| 2124 | `:wat::core::Record` | `Some(Record)` | category root |
| 2153 | `:wat::holon::Record` | `Some(HolonRecord)` | category root |
| 2200 | `name,` (inventory drain) | n/a — not a literal | codegen |
| 2447 | `name: format!(":wat::runtime::{}", variant)` | n/a — not a literal | codegen loop |

Every string-literal Aggregate name is `Some`. The wall asks `Nature::from_root_keyword`; it does not carry a list of root names (STOP-1).

## Row 5 — the wall REFUSES a non-root

Planted `register_builtin(TypeDef::Aggregate(AggregateDef { name: ":wat::probe::NotARoot" …}))` next to Struct.

```
no_hand_written_type_floor::aggregate_literals_are_only_category_roots ... FAILED
hand-written `TypeDef::Aggregate` literal(s) whose name is NOT a category root
(Nature::from_root_keyword is None) at [(941, ":wat::probe::NotARoot")].
```

**RED at types.rs:941**, naming the site. Plant reverted. `alias_literals_are_only_the_named_floor` and `category_roots_are_present_and_admitted` stayed green through the sabotage — a wall that refuses a non-root did not forget the roots.

## Row 6 — the wall ADMITS a root

The three category roots are present as Aggregate string-literal names and the lint is **green** (both before sabotage and after revert). `category_roots_are_present_and_admitted` asserts each of

```
:wat::core::Struct
:wat::core::Record
:wat::holon::Record
```

is in the scan **and** `Nature::from_root_keyword` is `Some`. A wall that refused everything would fail this test. That list is the non-vacuity population (row 6), not the admission rule (row 4 / STOP-1).

`:wat::kernel::Peer` is a fourth nature (`from_root_keyword` → `Some(Peer)`) but it is `register_builtin_leaf` — membership without structure (STOP-5). Not an Aggregate literal, not this wall's door.

## Row 7 — the wall asks the ORACLE, not a list

`tests/lint/no_hand_written_type_floor.rs` `use`s `wat::types::Nature` and calls `Nature::from_root_keyword(name)` on every Aggregate string-literal name. No hand-list of root names in the admission arm. The alias floor is a short match (`:wat::core::nil` only) whose reason the failure message prints — there is no alias oracle.

## Row 8 — zero exemptions added to pass

`grep -c 'allow\|rune:' tests/lint/no_hand_written_type_floor.rs` = **0**.

## Row 9 — floor (ORCHESTRATOR)

Not run.

## Row 10 — clippy (ORCHESTRATOR)

Not run.

## Row 11 — the named floor is ARGUED

Every surviving Rust type literal answers "why is this in Rust?":

### Impossible in principle (the floor)

**`:wat::core::Struct`** — what `defstruct` *produces*. Declaring it with `defstruct` is the concept declaring itself. Nature root; wall admits it because `from_root_keyword` is `Some`.

**`:wat::core::Record`** — what `defrecord` produces. Same impossibility.

**`:wat::holon::Record`** — the holonic-record nature root. Same impossibility as its sibling.

**`:wat::core::nil`** — STOP-2. The unit the language is built out of is `TypeExpr::Tuple(vec![])`, not a path. A wat `typealias` whose expr is `:wat::core::nil` only becomes `Tuple([])` via the canonicalize special-case that **exists because nil is already the unit** — the concept declaring itself. `:()` as the expr is the retired spelling the `BareLegacyUnitType` walker steers **away** from. Not "not moved yet".

### Not literals (codegen; STOP-3)

**Inventory drain (~2200).** `for schema in inventory::iter::<EdnSchema>()`; `name` is computed then used as field shorthand `name,`. The wall keys on a string-literal `name: "…"`. It did not see this site. No exemption.

**Runtime-error variant loop (~2447).** `name: format!(":wat::runtime::{}", variant)` inside `for (variant, coords) in variants`. Same: no string-literal name, so the wall does not treat it as a hand-written type home. No exemption. The wall could tell them apart structurally; STOP-3 did not fire.

### Moved this stone (no longer survivors)

`:wat::holon::BundleResult`, `:wat::holon::Holons`, `:wat::core::Bytes` — wat is the source of truth.

### Out of scope, named so they are not "missed"

`register_builtin_leaf`'s scalar primitives (STOP-5) — a different door, a different category. Folding them in would make a red ambiguous.

13 records already wat-sourced via `wat_record_from!` — not new work.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 wall carries a hand-list | **held.** Admission is `Nature::from_root_keyword`. |
| STOP-2 nil cannot be a typealias | **fired as designed.** nil stays the named alias floor with the circularity as its printed reason. |
| STOP-3 generated caught by the wall | **not taken.** Inventory `name,` and loop `name: format!(…)` have no string-literal name; the wall skipped them structurally. |
| STOP-4 moved alias changes shape | **held.** Stdlib load via `--check` of an H-2 probe was green; Equivalent no-op'd. |
| STOP-5 scalar leaves | **not taken.** Leaf door untouched. Peer is a leaf, not an Aggregate. |
| STOP-6 exemption to pass | **not taken.** `allow\|rune:` = 0. |

## Targeted checks (executor)

```
cargo test --release --test lint -- no_hand_written_type_floor
  3 passed (pre-sabotage)
  1 failed / 2 passed during sabotage (aggregate_literals RED at 941; roots still admitted)
  3 passed (post-revert)
cargo test --release --test lint -- no_hand_written_enumdef wat_record_from_sources
  4 passed (J's wall + load-set intact)
cargo test --release --test types -- option_and_result_are_registered_parametric probe_arc296_h2 probe_arc296_j
  5 passed
target/release/wat --check tests/types/probe_arc296_h2__variant.wat
  green (stdlib loaded; Equivalent held)
wall sabotage: RED at types.rs:941 naming :wat::probe::NotARoot, then reverted
```

## Files (this stone)

```
crates/wat-source-derive/src/lib.rs          + wat_alias_register_from!
wat/holon.wat                                + BundleResult, Holons typealiases
wat/core.wat                                 + Bytes typealias
src/types.rs                                 3 Alias literals → wat_alias_register_from!
                                             nil + 3 category roots annotated NAMED FLOOR
tests/lint/no_hand_written_type_floor.rs     NEW — sibling of J's enum wall
```
