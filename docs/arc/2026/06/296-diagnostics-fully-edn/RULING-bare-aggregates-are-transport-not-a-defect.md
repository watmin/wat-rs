# RULING — a bare aggregate is a TRANSPORT, and no ban is necessary

**Builder, 2026-08-15, verbatim:**

> *"aggregates are legal in their bare form - they exist to be a transport - they do not know what
> they hold, the caller who provides it and the reciever who operates on must be in agreement...
> so... it looks like no such ban is necessary here..."*

## WHAT IS RULED

`:wat::core::Struct` · `:wat::core::Record` · `:wat::holon::Record` in a slot are **legal**. They are
not a failed type — they are a **transport marker**: *"an aggregate of this holder crosses here; its
concrete type is a contract between the caller and the receiver, not a claim this slot makes."*

**The `reject_any` holder-root ban is VOID.** Not deferred, not blocked, not pending `EdnValue` —
**void**. It was aimed at a legal construct. It was never armed; the patch is discarded.

## WHY `AGGREGATE-MODEL` PRINCIPLE 5 IS SUPERSEDED ON THIS POINT

Principle 5 says:

> *"A bare holder root in a `[x <- …]` slot is an **Any** — its base state has nothing, so you can do
> nothing with it; it is FORBIDDEN. (`[r <- :wat::core::Record]` does not exist…)"*

**The premise is false.** There is exactly one thing you can do with a bare aggregate, and it is the
entire point: **carry it.** Principle 5 read "offers no accessors" as "is useless" and missed that
carrying is a use — the one the corpus actually needs.

And the deeper error, which is the same one arc 296 spent the day removing one layer up:
**transport is a property of the SLOT, not of the VALUE.** A record is a record whether it sits in
`user.program` or in a precisely-typed field. Principle 5 tried to make a *positional* concern into a
*type* prohibition.

**This also settles the "do we need a nature of transport?" question: NO.** `Nature` answers *"what
is this value?"* and travels with the value. Transport answers *"what does this slot promise?"* A
fifth holder would put a slot property on the value — the identical category error that registered
the Record umbrellas as `Nature::Struct` (see
`NOTE-the-nature-roots-are-registered-as-structs.md`). Doing it again, in the opposite direction,
would not be an improvement.

## THE EVIDENCE — every live site is deliberate, and says so

23 live (non-comment) sites across 4 files. The ones read in full:

- **`wat/program.wat:46`** — `user.program <- :wat::core::Record`, documented at the site:
  *"the user-extension slot, typed `:wat::core::Record` (the root — any record fits as a subtype);
  default `:wat::program::EmptyEnv`. NOT `wat.*` — user data."* Arc 259, The Forced Hand.
- **`wat/spawn.wat` ×3** — `init-fn` lambdas satisfying `:Fn()->wat::core::Record`, the contract that
  carries the above.
- **`wat/rete.wat` ×18** — facts held opaquely. Principle 7's own text: *"rete uses facts opaquely —
  as map keys, stored, reflected via `type` — never by named field."*

Zero accidental uses found.

## THE HALF THAT IS REAL — the receiver, not the type

The transport contract has two ends. Measured:

| | |
|---|---|
| **transport** — "some aggregate of this holder passes here" | **built**, correctly, at every site |
| **receiver declaration** — "I am receiving *this* concrete type" | **built in rete** — `defclause` + `value_matches_type_by_name` (`runtime.rs:7112`) dispatching on the value's CONCRETE class · **ABSENT for `user.program`** |

**Zero `.wat` files anywhere read `user.program` back out** — no accessor, no clause, no typed
receive. Rust writes it (`invoke_user_main_with_program`); nothing in wat receives it. That is a
half-built feature, and it is the honest defect this whole thread was circling. It is **not** a
missing type.

## WHAT THIS RULING COST TO REACH — the pattern, recorded once

Four positions held and abandoned in one stretch, each from reading a design sentence instead of
measuring:

1. *"`program::Env` must become a surface"* — ruled out the surface option by assertion (never
   checked whether accessors are read), then declared it the answer, then found arc 259 superseded it.
2. *"Mint `EdnValue`"* — built a new top from ONE line of prose, misattributed to `294.d` (which is
   the wire-kill stone) four separate times. Reverted.
3. *"The 18 rete sites are pre-ruled, migratable now"* — from principle 7's one sentence;
   `wat/rete.wat:1181` says something more specific and partly contradictory about those same slots.
4. *"Principle 5 must be enforced"* — the premise of everything above. Now void.

**The through-line:** a design sentence read as current, acted on without a measurement. Every
conclusion that held up this session came from imposing a check and reading what screamed; every one
that collapsed came from believing a document. `SCVTVM IDEM INDEX` — three sources, and the disk
outranks the design.

## STILL OPEN — carried, not closed

- **`user.program` has no receiver.** Zero wat consumers. Needs the rete pattern (a `defclause`
  keyed to the user's concrete env type) or an explicit ruling that the slot is Rust-only.
- **`wat/rete.wat:1181` comment vs code.** The block states — *"confirmed empirically"* — that a
  `defclause` declaring `fact <- :wat::core::Record` can NEVER match at runtime; the `defclause`
  directly beneath declares exactly that, twice, in code the floor proves works. One is wrong.
  Unresolved, and NOT to be migrated until someone who knows the intent reads it.
