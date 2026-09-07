# DESIGN — the outcome crosses, the resource stays

**Every retry ladder returns a nullary-payload closed enum whose `Exhausted` variant the match must
name; the peer and reply ride beside it. The two ladders that still stop after three flat tries get
the vis-bounded backoff the ack path already has.** `wat-scripts/fanout/circuit.wat` only.

Third draft. The invariant has not moved once. **The mechanism is no longer mine** — it is the
construction grok ran green at n=12 on the previous strike.

## ⛔ THE RULE THAT KILLED THE FIRST TWO DRAFTS, STATED AS THE REASON

```
containment rule (arc 293.W.2b): :wat::enum::Pure enum may only hold pure variant fields —
field "peer" has impure type (:wat::kernel::Peer :- […]), which CANNOT BE RECONSTRUCTED FROM
EDN BYTES ACROSS AN ADDRESS-SPACE BOUNDARY. Declare the enum :wat::enum::Impure if it must
hold a live resource (it then STAYS IN SHARED MEMORY AND NEVER CROSSES).
```

Reproduced on my own probe. `wat/service.wat:207` says the same thing from the other side:

```
:ephemeral        what I carry        (the body: resources + peer clients; NEVER CROSSES)
```

★★★ **This is not a type-system quirk to route around. It is the distributed fact.** A `Peer` is a
live handle. Drafts one and two put a live resource inside a value that must cross into a process
child, and the substrate refused three times in three different voices — Pure containment, unknown
constructors in the child, and surface S4c.

⚠ Both earlier drafts cited `:ephemeral [store <- Peer …]` as precedent. That is the field group
whose *definition* is "never crosses." **The exemplar proved the opposite of the claim.**

## ⛔ THE ONE CONTRACT DECISION — the outcome crosses; the resource stays beside it

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got       []
  :Exhausted [attempts <- :wat::core::i64])

(:wat::core::defenum :fanout::QueueRetry :wat::enum::Pure
  :Got       []
  :Exhausted [attempts <- :wat::core::i64])
```

Each retry helper returns `(Tuple <outcome> <peer> <reply-or-unit>)`: the **enum** is the part the
match must name, and the **peer** — a live resource — stays a plain local binding that never enters
a crossing value.

★★★★ **The invariant never needed the payload in the variant.** *Exhaustion must be a variant the
match has to name* is satisfied exactly as well by `:Exhausted [attempts]`. The peer sat inside the
variant only because the existing code returns `(peer, retry?)` tuples and I mirrored the habit —
and that habit is what collided with the boundary three times.

**Measured, not proposed:** grok ran this construction on the previous strike. Keyword arms matched
inside the EmptyEnv child and n=12 completed —
`total=24;distinct=24;dup=0;check-exhausted=0;mark-exhausted=0;ack-retries=0`.

## WHY — three hand-rollings of one shape, two of them dropping the flag

| ladder | line | on exhaustion | consequence |
|---|---|---|---|
| **check** | `:494-518` | `(Tuple 1 0)` → `gb-tick 1` → **`gave-back`** | **the terminal event.** Batch never acked; `vis` is 10¹² ns, so it is stranded past any bound |
| **mark** | `:560-576` | `second mm3` **discarded — no counter** | delivered-but-unrecorded; a later redelivery is a **duplicate** |
| **ack** | `:606-692` | vis-bounded backoff, `ack-retries` | fixed; proven live by `ack-retries=5` |

Measured twice on `2000 4 3 8192 true`: `gave-back == stranded batches`, `unacked == gave-back × 10`
(`4 → 40`, `1 → 10`). `:518` is the **only** producer of `gb-tick = 1`, so that counter has always
measured exactly one event — and **nothing is given back.** It is renamed.

## WHAT THIS IS AND IS NOT WORTH

★ **Gate is binary and failing in both our hands:** `2000 4 3 8192 true` completes, or it does not.

⚠ **Two rungs.** The enum is the rung where the mistake cannot be *expressed*; the backoff fixes the
instances. Enum alone changes nothing; backoff alone leaves the flag droppable.

⚠ **Does NOT fix the class.** The backoff formula stays in three copies (`:1204`, `:1540`, `:675`)
with `100` hardcoded in two, because an `EmptyEnv` child cannot see the file's own `defn`s.

⚠ **Does not explain the 25 % slope** (4348 → 3244), measured with `gave-back=0` on every point.

## OUT OF SCOPE — REJECTED

- **A `Peer` inside the outcome.** Refused by arc 293.W.2b, and rightly: a live resource cannot
  cross an address-space boundary. Three drafts died here; it is written down so a fourth cannot.
- **An `Impure` script-level enum.** Constructors are unknown in the child; the match goes `Open`.
- **Carrying `Seen::Reply` on the Worker surface.** S4c requires declaring the whole Seen protocol
  on Worker.
- **Hash-destructure arms.** `MatchShape::Open` determines no variant shape and tracks exhaustiveness
  by wildcard alone — **exactly as droppable as the `bool`**, and worse for looking exhaustive.
- **One generic `defn` in `wat/`.** The real fix for the duplication, and **the builder's ruling on
  the precedent's own terms** — *"promoted when it demonstrates excellence, never a side effect."*
  This stone is how that gets earned.
