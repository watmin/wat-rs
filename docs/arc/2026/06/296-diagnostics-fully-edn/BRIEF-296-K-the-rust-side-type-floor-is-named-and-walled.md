# BRIEF — 296 K: records, structs and aliases follow; the floor becomes named and walled

> Read `DESIGN-STONE-K-the-rust-side-type-floor-is-named-and-walled.md` first. Stone J is the worked
> example (`SCORE-296-J-…`) — same move, but J's wall admitted NOTHING and K's must admit exactly
> the category roots.

## THE WORK

1. **Build `wat_alias_register_from!`** — the third sibling of `wat_record_from!` (`:389`) and
   `wat_enum_register_from!` (`:580`) in `crates/wat-source-derive/src/lib.rs`. It reads a
   `(:wat::core::typealias :ns::Name <type-form>)` and emits the `TypeDef::Alias` row.
2. **Move the movable aliases** into their family `.wat` files:
   `:wat::holon::BundleResult` and `:wat::holon::Holons` → `wat/holon.wat`;
   `:wat::core::Bytes` → `wat/core.wat`. `:wat::core::nil` is STOP-2.
3. **Wall both literal shapes** in `src/types.rs`:
   - `TypeDef::Aggregate` — admitted **iff** `Nature::from_root_keyword(name).is_some()`.
     **ASK THE MAP; DO NOT CARRY A LIST.**
   - `TypeDef::Alias` — refused, except a short floor whose every entry carries a printed reason.
4. Reuse J's lint file where it fits; a sibling file is fine if that reads better. Say which and why.

## READ IN ORDER

```
tests/lint/no_hand_written_enumdef.rs   J's wall — the model, including its sabotage proof
src/types.rs:244-266                    Nature::root_keyword / from_root_keyword — "the single
                                        canonical keyword→nature map". THE ORACLE the wall asks.
src/types.rs:928 · 2123 · 2148          the three CATEGORY ROOTS (fields: vec![])
src/types.rs:2195 · 2442                the two GENERATED aggregate registrations — :2442 is a LOOP
                                        over runtime variants. Judgment: are these literals at all?
src/types.rs:961 · 986 · 1040 · 1073    the four Alias literals
crates/wat-source-derive/src/lib.rs:389 wat_record_from! — the shape to copy for the alias sibling
crates/wat-source-derive/src/lib.rs:95  binder_vector — the crate's ONE `:-` recogniser (H-3/J)
wat/cache.wat · wat/sqlite.wat          live `(:wat::core::typealias …)` declarations, for surface
```

## STOP TRIGGERS

- **STOP-1 — the wall carries a hand-list for aggregates.** The oracle exists
  (`Nature::from_root_keyword`). A list is the rung below and it rots; if the oracle genuinely
  cannot answer, STOP and report why rather than falling back to a list.
- **STOP-2 — `:wat::core::nil` cannot be spelled as a `typealias`.** It is `TypeExpr::Tuple(vec![])`,
  the unit the language is built out of; a bootstrap root is a REAL possibility. Do not force it.
  Report what refused it, and it joins the named floor with that refusal as its reason.
- **STOP-3 — a generated registration (`:2195`, `:2442`) is caught by the wall.** They are codegen,
  not hand-written literals. If the wall cannot tell them apart structurally, STOP and report —
  a wall that needs an exemption for generated code on day one is drawn at the wrong level.
- **STOP-4 — a moved alias changes shape.** `Bytes` is `(Vector :- [u8])`, `Holons` is
  `(Vector :- [HolonAST])`. Byte-equivalence is enforced by the loader (`Existing::Equivalent`); a
  load failure means the transcription is wrong, not the gate.
- **STOP-5 — the scalar leaves drag in.** `register_builtin_leaf`'s four primitives are a different
  door, affirmatively out of scope.
- **STOP-6 — the wall lands with an exemption you added to make it pass.** Same as J's row 11.

## WHAT MAKES THIS THE PRECEDENT

J proved a class can be purged when nothing legitimately survives. K proves the harder case: when
something DOES survive, its survival is **argued and structurally enforced**, so "cannot be wat" and
"has not been moved yet" stop looking alike. Every future reader of a Rust type literal gets an
answer instead of a residue.
