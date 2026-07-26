# NOTE — the 293.W containment wall is BLIND to Rust opaques: a record can hold a live resource

> **Found by probe 2026-07-25** (arc 278, grounding parametric records for cache Stone 2). Builder:
> *"did you find a legit flaw in our enforcement?"* — yes. *"i'm not chasing it now"* → recorded, not fixed.
> Recorded per the arc-109 `NOTE-*.md` convention.
>
> **Possibly material to 293's CLOSURE GATE.** That gate says *"the only place these aggregates differ is when
> they are being passed around."* Passed-around IS the portability axis, and this note is a hole in the wall
> that governs it — so the audit that gates 293's closure may need to cover opaque fields, not just
> record-used-as-struct.

## The wall is SOUND; its ENROLLMENT is not

`validate_aggregate_containment` (`src/check.rs:14213`, called from `freeze/env.rs` after all types register)
is real and it works. Controls, run this session:

- `(defrecord :probe::Direct [w <- :wat::io::IOWriter])` → **REJECTED**, exit 3:
  `#wat.type/ImpureFieldInPureAggregate` — *"pure aggregate `:probe::Direct` may only hold pure fields — field
  `w` has impure (struct) type `:wat::io::IOWriter`."*
- `(defrecord :probe::Holder [b <- :probe::Box<wat::io::IOWriter>])` → **REJECTED**, and it names the full
  parametric type. **The wall sees through type parameters.**

## The gap — `is_pure_type`'s opaque knowledge is eight hardcoded strings

`is_pure_type` (`src/check.rs:14097`) is sound for wat-declared types: `Nature::is_pure()` for aggregates
(Struct/Peer → impure), the declared `:wat::enum::{Pure,Impure}` marker for enums, recursive over
containers/newtypes/tuples, `TypeExpr::Var(_) => false`. But for **Rust opaques** it consults a hand-written list:

```rust
"wat::kernel::ChildHandle" | "wat::io::IOReader" | "wat::io::IOWriter"
| "wat::holon::OnlineSubspace" | "wat::holon::Reckoner" | "wat::holon::Engram"
| "wat::holon::EngramLibrary" | "wat::holon::Hologram" => return false,
```

Anything absent falls to `None => true` — *"portable by convention"*, the arm written for formal type
parameters. **So every `#[wat_dispatch]` opaque minted since that list was written is invisible to the wall.**

## Proven — all exit 0 where they must be exit 3

- `(defrecord :probe::Direct [c <- :wat::sqlite::Connection])` — a live, thread-owned sqlite handle.
- `(defrecord :probe::Raw [c <- :rust::sqlite::Connection])` — the raw path, proving the `Alias => true` arm is
  not what masks it (`wat/sqlite.wat:42` typealiases the wat name onto the rust one).
- `(defrecord :probe::Smuggle [c <- :wat::cache::Lru<String,i64>])` — **the cache Stone 1 primitive**
  (`a86f521c`), shipped the day before this was found.

## Why it matters

293.W was built *from a grounded breach* — a Struct nested in a Record crossing a process peer (`#w/S {:a 99}`
reconstructed far-side) — to make the wire wall a **TYPE guarantee** (`AGGREGATE-MODEL.md` § principle 8). For
opaques that guarantee does not hold. A record that claims to be EDN can contain a live resource, and the
containment rule's own error text states why that must never exist: *"a record holding a struct field could
never cross — it must not exist."*

Severity is *latent, not observed*: it requires someone to declare such a record. But the type system currently
permits it, which is precisely what 293.W set out to make impossible.

## The fix (deferred — two rungs)

- **Narrow, available now, needs nothing from arc 255:** enroll a `#[wat_dispatch]` opaque's purity at the
  moment it registers its type, and DELETE the eight hardcoded names. Same shape as arc 296's `EdnSchema`
  inventory drain — structure is the schema; a hand-maintained parallel list drifts.
- **Root:** arc **255**'s builtin registry — opaques declare purity where they register and `is_pure_type`
  *projects* from it. See `255/NOTE-purity-is-definition-time-queryable-metadata.md`, where this is recorded as
  the **third instance** of the hand-list-drift class (alongside `is_pure_total` and `src/rete/purity.rs`), and
  the only one that is a correctness wall rather than a tooling gate. **255 is PARKED** pending prerequisites
  (builder, 2026-07-25), so the narrow rung is the near-term path.

## Unchased — do NOT inherit as fact

Whether `:rust::sqlite::Connection` is **absent** from the `TypeEnv` entirely, or **present with a pure-reading
nature**. That determines whether the fix is "register it" or "register it correctly." Ground it before drawing.

## Blast radius warning

If opaques begin reading impure, **every record currently holding one goes RED at startup.** That is the point —
293.W's containment pass caught six real stdlib mis-declarations when it landed and was called a design oracle —
but it is a cascade, not a one-liner. Expect the same shape: real mis-declarations surfacing as the wall closes.

## Status

**DEFERRED**, builder-ruled 2026-07-25 (*"i'm not chasing it now"*). Grounded: `src/check.rs:14097`
(`is_pure_type`), `:14213` (`validate_aggregate_containment`), `wat/sqlite.wat:42` (the typealias),
`src/types.rs:202` (`Purity`), `Nature::is_pure()`. All three violations and both controls were run by the
orchestrator's own hand.
