# 293.W — the deep wire wall: the holder's comms boundary, made a TYPE guarantee

> **Status: SCOPED (2026-06-29). The PRIORITY** — pulling the projection-depth thread surfaced a grounded breach of
> §7 / R3. Builder: *this IS core 293* (the holder's categorical comms boundary). Gates K3-revise + K5.
> Pairs `AGGREGATE-MODEL.md` § principle 8 (the CONTAINMENT RULE) + § `to-record` (the projection that depends on it).

## The breach (grounded this session)

`is_portable_type` (`check.rs:13543`) checks a record's portability by its **top holder only** — `Some(Aggregate(a)) =>
a.holder.is_portable()` — and never recurses into field types (it *does* recurse for `Tuple` / `Vector<T>` / `Newtype`;
aggregates + enums are the holdouts; the enum arm carries an explicit exigere-violating `"not yet enforced"`). So a
`Record` carrying a `Struct` field passes the wall, and the runtime codec serializes the struct into a tagged map.
**Proven live** (the disconfirming probe): a child built `(:w::R 7 (:w::S 99))` (a record with a struct field) and the
parent `recv'`'d it across a process peer — `#w/S {:a 99}` reconstructed on the far side. A `Struct` crossed comms.
§7 (*"a Struct crosses NO comms, ever"*) and R3 *SUB SUPERFICIE QUOD ES* (*"the holder is enforced HARD … the same
leak class as a struct … crossing the wire"*) are both violated. R3 is PROBANDUM until this lands.

## The cure — the CONTAINMENT RULE (the top rung, not a runtime patch)

A non-portable field cannot be **reconstructed** from EDN bytes on the far side (you cannot materialize a bound socket —
there is no default value). So a portable container that held one could never be reconstructed → **it must not exist.**

> **A portable aggregate (`Record` / `HolonRecord`) may declare ONLY portable field types.** A `Struct` field is
> ILLEGAL at type declaration. A `Struct` itself still holds anything (in-locus — sockets, caches, nested structs).

This makes *"a struct crosses NO comms"* a **structural guarantee**: a record cannot *hold* a struct → can never
*carry* one across. The illegal state has no form (extirpare's top rung). `is_portable_type` staying shallow then
becomes *correct* (the rule guarantees the depth), and `to-record`'s recursive strip is well-defined (kept fields are
portable by the rule).

## The contract (pinned)

> ⊹⊹ **SCOPE CORRECTED (2026-06-30, builder) — THE WIRE WALL IS PURELY COMPILE-TIME; the runtime checks RETIRE.**
> The job is ONE sentence: *the compiler won't let you write code that reads or writes a struct over non-thread
> memory.* It is NOT trust-boundary / bad-bytes defense — deserialization of untrusted input is the user's validation
> problem (every language punts it to the caller), explicitly OUT OF SCOPE. Builder: *"this is a user problem — they
> gotta validate their inputs — we are not solving that … we are solving 'you cannot read or write structs from
> non-thread memory' — the compiler doesn't let you write code that does that."*
>
> The wall is **THREE compile-time rules, ZERO runtime code:**
> 1. **W.1 — aggregate containment** (a record can't HOLD a struct field; the declaration gate, item 1 below). LANDED (`ff29f135`).
> 2. **2b — THE ENUM MOBILITY MARKER** — enums **DECLARE** `:wat::enum::Portable` | `:wat::enum::Anchored`; the
>    predicate **reads the declaration** (`e.mobility.is_portable()`, mirroring the aggregate's `holder.is_portable()`);
>    an enum-containment pass enforces it. The predicate the rules consume. **DESIGN SETTLED + ratified 2026-06-30 —
>    see § 293.W.2b below** (supersedes the earlier "the enum arm recurses / derived" framing, which was four-questioned
>    out: a *derived* predicate masks intent — the same reason surface `:holder` is mandatory).
> 3. **2d — PEER-TYPE CONTAINMENT** (the W.1 rule lifted to the peer): a wire peer (`Process'`/`ConnPeer'`) may not be
>    TYPED with a non-portable `I`/`O`. Then the ORDINARY type checker forbids struct-on-wire — `send'(peer, struct)`
>    is a `struct ≠ portable-I` unify error; `recv'(peer)` can never produce a struct (its `O` isn't one); the "read
>    into a struct off a wire peer" call path **has no form.** No special send'/recv' gate needed.
>
> **RETIRED into 2d** — the 293.W.2a runtime guards (`decode_trusted_wire` reject `fe012223` + `reject_non_portable_on_wire`)
> AND the 293.W.2c send'-site gate (`7a040b0e`). They were the correct INTERIM enforcement (held the line + caught/proved
> the breach while no compile wall existed); once peer containment is total, a struct can never reach the wire from any
> wat program → the runtime guard defends a door no wat code can walk through: DEAD (send' side) / OUT-OF-SCOPE (decode =
> bad bytes). **DELETE both in the 2d strike.** The wall ends as zero runtime code.
>
> So below: **item 1 (declaration gate) STANDS; item 2 (recv' backstop) RETIRES** (it was bad-bytes defense); **item 3's
> predicate-completion is 2b** — but 2b's SHAPE is now the **enum mobility marker** (§ 293.W.2b), NOT recursion + a rune.
> The recursion-as-predicate and the `rune:lint` on service enums are both **SUPERSEDED** (a `Portable` enum *declares*
> its capability and a containment pass enforces it — no inference, no rune; and grounding found **no live enum carries
> a direct `Receiver<T>`** anyway — the cited `StdOutService::Event` died when the services rehomed to Rust). The new
> keystone is **2d peer containment.**

1. **Declaration gate (the core):** registering a `Record` / `HolonRecord` aggregate whose any field type is
   non-portable is a **hard declaration error** (`MalformedDecl` / a typed error in `register_types` / the aggregate
   registration path). "Non-portable field type" = `is_portable_type(field_ty) == false`. (Reuse the existing
   `is_portable_type`; it is the right predicate, just newly *enforced* at declaration instead of only consulted at
   `send'`.) A `Struct` aggregate is unrestricted.
2. **The `recv'` backstop (the untyped top-level path):** `recv'` (`eval_peer_recv_prime`, `runtime.rs:24685`) refuses
   to reconstruct a **bare top-level `Holder::Struct`** value off the wire — the one path the declaration gate can't
   reach (a child `pprintln`s a bare struct, no type to check). A struct shall not *arrive*.
3. ⊘ **SUPERSEDED (2026-06-30) by § 293.W.2b — the enum mobility marker.** *(Original framing, preserved:* the enum arm
   of `is_portable_type` recurses into variant field types; genuinely-non-portable service-control enums carry a
   `// rune:lint(<lint>) — <reason>`; the deferral comment dies.*)* The recursion-as-predicate was four-questioned out
   (derived → masks intent); the rune is moot (grounding found no live enum carries a direct `Receiver<T>`). REPLACED by:
   enums **declare** `:wat::enum::Portable` | `:wat::enum::Anchored`, the predicate reads the declaration, an
   enum-containment pass enforces it. The deferral comment dies either way. **See § 293.W.2b.**

## RED probe

`tests/types/probe_arc293_W_containment.{rs,wat}` — a record declaring a struct field is REJECTED at load:
```clojure
(:wat::core::defstruct :w::Conn [fd <- :wat::core::i64])
(:wat::core::defrecord :w::Bad [tag <- :wat::core::i64  c <- :w::Conn])   ; ILLEGAL — a record cannot hold a struct
```
RED at HEAD: this loads cleanly today (the breach). GREEN after 293.W: the load FAILS with a containment-rule error
naming the offending field. (A second `_bad`-style probe asserting the breach roundtrip now errors is a follow-on once
the gate lands.)

## Blast radius (the existing illegal declarations to surface + fix)

Enforcing the rule will RED any current `Record`/`HolonRecord` that declares a struct field — the corpus must be swept
(each is either a real bug → fix, or a struct-that-should-be-a-struct → the *container* should be a struct). The
breach probe's `:w::R` is the synthetic case; the gate run reveals the real ones (a cascade — normal, the meter to
zero). Service enums with `Receiver<T>` → runed. **Weigh forced-clean; the cascade is the progress meter.**

## Decomposition
- **293.W.1 — the declaration gate** + the `recv'` backstop + the RED probe → GREEN. **LANDED (`ff29f135`).**
- **293.W.2b — the enum mobility marker** (`:wat::enum::Portable` | `:wat::enum::Anchored` + containment pass +
  corpus migration) — the predicate the rules consume, made total by *declaration*, not recursion. **§ 293.W.2b below.**
- **293.W.2d — peer-type containment** (the keystone; deletes the interim 2a/2c guards; makes `make-channel`
  tier-aware → closes the `:svc::Request` thread-channel case).
- **293.W.2e — `address-wire?`** (this stone). The Address answers "is this shared memory?" The
  runtime already knows (`portable_form`). Wat did not. First mouth. Type still lies — that is 2f.
- **293.W.2f — Address type stops lying** so a process `bracket/map` of a thread handle is
  a CheckError (Shared ↛ Wire). Live MCP 2026-08-16: `address-wire?` was false and the
  checker still accepted the circuit. See `DESIGN-STONE-293.W.2f-process-may-not-dial-shared.md`.
- Then **K3-REVISE** (annihilate `to-struct` + `$struct`; the pair) → **K5** → showcase graduates.

## § 293.W.2b — THE ENUM MOBILITY MARKER (DESIGN SETTLED + builder-ratified 2026-06-30)

> ⊹⊹ **SUPERSEDED → PURITY (2026-06-30, builder: *"a wonderful finding … our next priority"*).** The marker is
> **`:wat::enum::Pure` | `:wat::enum::Impure`** (Rust **`Purity { Pure, Impure }`** on `EnumDef`), NOT `Portable`/
> `Anchored`. The axis is **PURITY** (the value holds nothing but data vs holds a live resource) — the *cause*; crossing
> the wire is the *consequence*. The long-term-stability bias renames the cause everywhere, in one change, so no seam
> survives: **`Holder::is_portable` → `is_pure`**, **`is_portable_type` → `is_pure_type`**, the wire wall → the **purity
> wall**, and the holder is understood as the purity axis refined (`Struct` permits impurity; `Record`/`HolonRecord`
> guarantee purity). `:wat::kernel::Failure` is pure data mis-declared `defstruct` → **`defrecord`** (the 2616-cascade
> root). One purity family with function effect-purity (`:wat::runtime::Purity` = `:Pure`/`:Effectful`) — shared `:Pure`
> root, domain-specific impure-poles. **THE CANONICAL STATEMENT is now `AGGREGATE-MODEL.md § THE PURITY AXIS.** The
> `Mobility`/`Portable`/`Anchored` content below is the PATH (the movement-frame, intueri-crowned, then renamed to the
> cause) — preserved, marked. Read it for the four-questions + the two intueri casts; read AGGREGATE-MODEL for the model.
>
> SUPERSEDES the "the enum arm recurses (derived)" framing above. Portability is **declared**, not inferred —
> mirroring how an aggregate declares its `Holder`. The model becomes uniform: **every portable container (record,
> holon, enum) = a DECLARED capability + a CONTAINMENT gate; `is_portable_type` READS the declaration; the gate
> (using the predicate) guarantees the fields honor it.** (Aggregate: reads `holder.is_portable()`, W.1 guarantees the
> fields. Enum: reads `mobility.is_portable()`, the enum-containment pass guarantees the variant fields. No special case.)

### The finding that forced it (a VALID, revealing breach — builder: *"this is a valid and reveal finding"*)
Completing the predicate (the first, *derived* cut: `is_portable_type`'s enum arm recurses) made `make-channel`'s 254.1
portability gate (`check.rs:10573`) reject `:svc::Request` (`wat-tests/service-template.wat:220`) — a request enum
carrying reply-`Sender`s in its `:Ack`/`:Get` variants. It had passed only because the enum arm returned a blanket
`true`; the recursion correctly exposed it as non-portable. The wire wall working as designed. `:svc::Request` is
genuinely `Anchored` (it holds channel handles — a `Sender` is not atomizable / not EDN-able, `value/value.rs:851`).

### The fork (four-questions) — DECLARED beats DERIVED
- **Derived** (`is_portable_type` recurses; enum portable iff every variant field portable): fails **Obvious** (intent
  invisible at the def — you chase the transitive field graph), **Simple** (a far-off `:Sender` silently flips the
  type's wire-eligibility), **Honest** (computes the result but can't express or enforce INTENT — the masked-intent
  problem the project already rejected when it made surface `:holder` mandatory). CUT.
- **Declared** (the enum declares its capability; a containment pass enforces): Obvious/Simple/Honest all hold, and it
  makes the predicate **non-recursive** (reads a declaration) → the cycle-guard the derived recursion needed is gone,
  and the cycle class evaporates.

### The name (intueri, double-cast, builder-ratified)
Not the `holder` — "an enum has a holder" is a category pun (the holder is the aggregate's BACKING; a sum has none).
The marker names a CAPABILITY, declared directly. **Round 1** (`:wire`/`:locus`) was rejected by the builder: process
and remote ARE loci, each with their own in-locus structs on the far side, so "locus" mis-says the axis. The true axis
is **shared-memory (thread tier, moves by reference, carries live resources) vs serializable-across-address-spaces.**
**Round 2** crowned:
- **`:wat::enum::Portable`** — values serialize to EDN and travel to another address space (process/remote). Overturns
  round 1's rejection of "portable": wat is proudly Linux-only (refuses portability-as-virtue), freeing the root sense
  "able to be carried"; AND the disk already uses "portable" for exactly this (`is_portable_type` ~30 sites +
  `Holder::is_portable()`, `types.rs:138`) → **one word for one capability, user-vocab == compiler-vocab.**
- **`:wat::enum::Anchored`** — may hold live resources (`Sender`/socket/closure); drifts on its line (moves between
  threads by reference) but cannot sail away (cross an address space). Beats `Pinned`/`Frozen` (over-claim total
  immobility, Rust `Pin`); **kills the builder's own seed `ThreadLocal`** as a Level-1 INVERSION (`thread_local!` =
  per-thread ISOLATED storage — the opposite of shared-across-threads).
- Namespace **`:wat::enum::`** is STRUCTURAL, not semantic: any semantic namespace contradicts one pole
  (`:wat::wire::Anchored` lies; `:wat::mem::Portable` lies); the neutral grouping puts the whole load on the member.

### The shape + the user forms
A **mandatory positional kind-word** right after the type name (no default — a default masks intent). Namespaced, so
unmistakable from the bare Capitalized variant keywords. Parse is positional-mandatory (slot 1 after the name MUST be
one of the two markers, else `MalformedDecl`).
```clojure
(:wat::core::defenum :order::Status :wat::enum::Portable  :Pending  :Paid [cents <- :wat::core::i64])
(:wat::core::defenum :svc::Request  :wat::enum::Anchored   :Push [v <- :wat::core::i64]
                                                           :Get  [reply <- :wat::kernel::Sender<svc::State>])
```
Internal mirror of `Holder`:
```rust
pub enum Mobility { Portable, Anchored }                                    // "how far can the value travel"
impl Mobility { pub fn is_portable(&self) -> bool { matches!(self, Mobility::Portable) } }
// EnumDef gains  mobility: Mobility  (mandatory);  is_portable_type enum arm → e.mobility.is_portable()
```

### The strike (re-shaped 2b — REPLACES the derived recursion)
1. `Mobility { Portable, Anchored }` + `is_portable()` in `src/types.rs` (mirror `Holder`).
2. `EnumDef.mobility: Mobility`; `parse_defenum` reads the mandatory positional marker (`types.rs:1868`; else `MalformedDecl`).
3. `is_portable_type` enum arm → `e.mobility.is_portable()` (the derived recursion + cycle-guard are REMOVED — the
   predicate no longer recurses through nominal types).
4. **Enum-containment pass** (parallel to W.1 `validate_aggregate_containment`, `check.rs:13686`): a `Portable` enum
   whose any variant field is non-portable is a hard declaration error; an `Anchored` enum is unrestricted.
5. Migrate the ~10 wat `defenum`s + 4 Rust builtin enums to declare mobility (most → `Portable`; the in-locus few —
   `ServiceEvent`, `StepResult` if it carries WatAST, etc. → `Anchored`). `:svc::Request` → `:wat::enum::Anchored`.
6. RED probe (revise `probe_arc293_W2b_enum_recursion`): a `Portable` enum with a non-portable variant field is
   REJECTED; a marker-less `defenum` is REJECTED; a record holding an `Anchored` enum is REJECTED.

### One thread that rides forward to 2d
`:svc::Request` becomes honestly `:wat::enum::Anchored`, but its `make-channel` (`service-template.wat:220`) is a
**thread**-tier channel — and the thread tier is wire-wall-EXEMPT (an `Anchored` value rides shared memory fine). So
`make-channel`'s portability gate must become **tier-aware** (a thread channel accepts `Anchored`; a process/remote
channel requires `Portable`) — that is **2d**'s tier-awareness (`ConnPeer'` vs `ThreadSelfPeer'`), where the
`:svc::Request` fixture reaches full green. 2b lands the marker + containment + migration; the thread-channel
exemption closes with 2d.

### Path of voices
Builder's: the holder-pun catch (*"an enum has a holder feels strange … abusing holder feels wrong"*); the loci
correction (*"process and remote /are loci/ … they can use structs on their far side"*); the namespaced-keyword form
(*":wat::enum::{...} … they look better"*); the seed `ThreadLocal`/`Portable`; the ratification. Apparatus's: the
four-questions (declared vs derived), the holder-symmetry, the two intueri casts. intueri crowned `Portable` (the
disk-grounded overturn) + `Anchored` (the moored-vessel precision) + `Mobility` (the Rust type), killing `ThreadLocal`
as an inverted Level-1 lie.

## Pairs
`AGGREGATE-MODEL.md` § principle 8 + § `to-record` · `CLOSE-SEQUENCE-293-294.md § THE SURFACE KIT` (the pivot banner) ·
`291/STRIKE-4b-struct-state.md` (R8 — the EDN wire wall, the soul/body line) · `feedback` exigere (the deferral) ·
the `rune:lint` exemption scheme (`4ce97de3`; excusare audits the reason).
