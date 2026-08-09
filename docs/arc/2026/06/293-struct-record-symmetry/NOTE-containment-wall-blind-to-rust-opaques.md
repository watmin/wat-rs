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

### ⚠ SHARPENED 2026-08-08 (arc 278, STOP-3) — for a PARAMETRIC opaque it is a DIFFERENT arm

The paragraph above names the `TypeExpr::Path` arm's `None => true`. That is right for the two
non-parametric sqlite handles. It is **not** the arm that admits `Lru<K,V>`: a parametric opaque never
reaches the Path arm at all. It lands in `TypeExpr::Parametric`, whose match on `head.as_str()` lists only
the kernel channel/peer heads and then falls through to

```rust
_ => args.iter().all(|a| is_pure_type(a, types)),   // "pure iff its TYPE ARGS are pure"
```

so the **container is presumed pure and only its type arguments are checked.** Proven by run this session
(`--check`, exit codes by hand):

| probe | verdict |
|---|---|
| `defrecord [c <- :wat::cache::Lru<String,i64>]` | **exit 0 — ACCEPTED** |
| `defrecord [c <- :wat::cache::Lru<wat::io::IOWriter,i64>]` | exit 1 — refused |
| `defrecord [c <- :rust::cache::Lru<String,i64>]` | **exit 0 — ACCEPTED** |
| `defrecord [w <- :wat::io::IOWriter]` (control) | exit 1 — refused |

The `IOWriter`-as-type-argument row is the discriminator: it proves the args ARE walked, so the miss is the
container, not the recursion. **This matters for the fix:** enrolling opaques only in the Path list would
leave every parametric opaque still admitted. Both arms need the same enrollment.

**And it is the same arm, patched before.** `check.rs:12868-12882` records `Peer<i64,String>` being judged
pure by this exact fallthrough and admitted into a pure Record — fixed 2026-08-03 by adding four names
(`Peer`, `Thread`, `Process`, alongside `ThreadSelfPeer`) to the hardcoded head list. `Lru` is the next type
standing in the identical hole. That is four hand-patches to one stem; the class is the list itself.

### ⚠ AND THE WALL DOES REACH `:durable` — proven, with both controls

Separately grounded this session, because the connection-scoped-world stone rested on it: a defservice's
`:durable` slot synthesizes `<svc>::Record`, a **pure aggregate**, so `validate_aggregate_containment` does
govern it. `:durable [w <- :wat::io::IOWriter]` → refused, naming `":probe::ctr::Record"`; `:durable [count
<- i64]` → accepted (non-vacuity). **293.W is a compiler-enforced wall, not a convention** — the enrollment
is the only hole.

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

> **2026-08-08 — STILL UNCHASED for the sqlite pair, and deliberately not guessed.** This session chased the
> PARAMETRIC case (`Lru`) and settled it: that one never consults `types.get` at all, so absent-vs-present is
> moot for it — the Parametric arm's arg-fallthrough decides it before the lookup. The sqlite handles are
> non-parametric and DO reach `types.get`, where `None => true` and `Some(Alias(_)) => true` are
> indistinguishable from the outside — both read pure. Which one fires is still unknown and still decides the
> fix's shape. Do not inherit either reading.
>
> **One thing that IS now grounded and bears on the fix:** `scope = "thread_owned"` — the exact fact the wall
> wants — is already DECLARED on the `#[wat_dispatch]` attribute (`src/rust_deps/cache.rs:60`). It is consumed
> only by codegen (how the handle is wrapped), never projected into the type environment. So the narrow rung
> does not need new information from the author; it needs the information already on the declaration to reach
> `is_pure_type`. `scope != "shared"` ⇒ impure is a candidate projection, **not yet ruled** — the three scopes
> are `shared | thread_owned | owned_move` (`crates/wat-macros/src/lib.rs:136`) and what `shared` should mean
> for purity is a real question, not an obvious one.

## Blast radius warning — ⚠ MEASURED 2026-08-08, AND IT IS EMPTY

The original warning read: *"If opaques begin reading impure, **every record currently holding one goes RED at
startup** … it is a cascade, not a one-liner."* **Measured, that is wrong for today's corpus.**

Every `#[wat_dispatch]` opaque in the tree is **three live families** — `:rust::cache::Lru`,
`:rust::sqlite::Connection`, `:rust::sqlite::ReadConnection` (plus `:rust::lru::LruCache`, the study-only
crate oracle slated for annihilation). A whole-corpus sweep for fields typed by any of them across
`wat/ wat-tests/ wat-scripts/ tests/ crates/` returns **18 sites, and not one is an illegal aggregate field**:

- **fn parameters** (the bulk) — legal; an opaque may be *passed*, it may only not be *held* by a pure aggregate.
- **2 `:ephemeral` slots** (`sqlite-store.wat:247`, `cache.wat:180`) — the correct placement, by intent.
- **1 aggregate that genuinely holds two opaques** — `:wat::cache::HolographicLru` (`cache.wat:254`) — and it
  is already a **`defstruct`**, correctly impure, with a comment saying exactly why.

**So enrolling the opaques would turn RED on zero existing sites.** The severity stands as the NOTE first
recorded it — *latent, not observed* — but the COST does not: this is a small fix, not a cascade, and the
"cascade" framing was the reason to defer it. Whoever re-poses the deferral should pose it with this number.

## Status

**DEFERRED**, builder-ruled 2026-07-25 (*"i'm not chasing it now"*). Grounded: `src/check.rs:14097`
(`is_pure_type`), `:14213` (`validate_aggregate_containment`), `wat/sqlite.wat:42` (the typealias),
`src/types.rs:202` (`Purity`), `Nature::is_pure()`. All three violations and both controls were run by the
orchestrator's own hand.
