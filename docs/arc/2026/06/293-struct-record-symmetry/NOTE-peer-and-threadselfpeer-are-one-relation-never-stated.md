# NOTE — `Peer'` and `ThreadSelfPeer'` are ONE RELATION, and it has never been written down

**Filed 2026-08-12 from arc 278, into arc 293 because 293.W.2d MINTED `ThreadSelfPeer'`**
(`BRIEF-293.W.2d-peer-purity.md`). Not a ruling — a grounded statement of a relation the substrate
currently enforces by *enumeration at every site* rather than by stating it once, plus the
measurement that surfaced it and the one hard guard any fix must keep.

## The two heads, and the line between them

| head | permits | who holds one |
|---|---|---|
| `ThreadSelfPeer'<S,R>` | **in-locus, ANY I/O** — the escape hatch for peers carrying impure values (reply-`Sender`s, live handles) | thread tier |
| `Peer'<S,R>` | **wire-safe, PURE I/O only** | process tier — **and every future remote locus** |

The line is **shared memory or not**, and it is deliberate. 293.W.2d's own brief states the
motive: *"the new `ThreadSelfPeer'<I,O>` (any I/O, in-locus) is the escape hatch for thread
self-peers that carry impure values … Then bare-`Peer'` `send'` is statically pure-safe; impure
peers are a distinct in-locus type."* `check.rs:9242` says it in the diagnostic the user reads:
*"(in-locus, shared memory), use `ThreadSelfPeer<I,O>` — any I/O types."*

## Where the line actually lives — NOT in `defservice`

`defservice` is **locus-blind by design**. The tier split is exactly two `extend-type`s of the
`Locus` surface in `wat/spawn.wat`:

```
wat/spawn.wat:451   (extend-type :wat::spawn::ThreadOpts  :wat::spawn::Locus)   ; "Thread (shared-memory) impl"
wat/spawn.wat:523   (extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus)
```

and the protocol's stated contract is *"A new transport joins as one `extend-type`, zero edit to
`start`"* (`spawn.wat:336`), *"the protocol builds the per-tier prog"* (`service.wat:1989`). **That
is the design that lets N remote loci (anon pipes · FS UDS · localhost tcp · mTLS · …) join without
touching `defservice`.**

The shared-memory line is also what decides **whether forms are transmitted at all**:

> `spawn.wat:450` — *"service-forms: thread arm ignores it (serve is already in the parent
> universe)."*

Shared memory ⇒ nothing to ship. Not shared ⇒ ship the closure. That is the whole reason form
transmission is a process-side concern only.

## The relation, stated (this is what is missing from the record)

A value meeting the **stricter** contract meets the **looser** one. `Peer'` is strictly stricter
(pure-only) than `ThreadSelfPeer'` (any I/O). Therefore, **with identical type args**:

```
Peer'<S,R>  is safely usable where  ThreadSelfPeer'<S,R>  is expected.     ← SAFE
ThreadSelfPeer'<S,R>  is NEVER usable where  Peer'<S,R>  is expected.      ← MUST STAY REFUSED
```

The second line is the mobility wall itself: it is what stops an in-locus peer holding live handles
from walking onto the wire. `check.rs:12986` already enforces that direction at the wire boundary —
*"Arc 293.W.2d — `ThreadSelfPeer'` is always in-locus (never wire-safe)"* — found, per its own
comment, *"by probe with a positive control."*

**The substrate has never stated the safe direction.** Instead it enumerates the pair, by hand, at
each site that must accept either — `check.rs:9835`, `:10159`, `:11091`, `poll'`/`select'`'s `self`
(the 293.W.2d brief itself prescribes *"accept BOTH `Peer'` and `ThreadSelfPeer'` for self"*), and
`:10176`'s error string. `grep -rn ThreadSelfPeer src/` returns ~24 lines across those sites. That
is one derivation implemented N times — the shape arc 109 keeps pulling out.

## The measurement that surfaced it (arc 278, 2026-08-12)

Arc 278 needs a **static** call to `serve` so a closure walk can root at it; today both tiers reach
`serve` through `(apply (keyword/from-string …))`, which no walk can follow. Probe: retype `serve`'s
`self` from `ThreadSelfPeer` to `Peer` (one line) and run the floor.

**Result: 4389 run, 1 FAILED** —

```
probe_arc209_c2_defservice_dispatch::defservice_generates_dispatch_loop_round_trips_on_thread
  :my::counter::serve: parameter #1
    expects :wat::kernel::Peer<my::counter::Status,my::counter::Admin>;
    got    :wat::kernel::ThreadSelfPeer<my::counter::Status,my::counter::Admin>
  at tests/services/probe_arc209_c2_defservice_dispatch.wat:76:36
```

**The red was the wall working.** Retyping `serve` at `Peer` asks a thread-tier
`ThreadSelfPeer` value to pass as a `Peer` — the forbidden direction — and the checker refused it.
Reverted.

What the probe also established, by reading the arms rather than their comments:

| | peer VALUE supplied | how it reaches `serve` |
|---|---|---|
| thread | `ThreadSelfPeer<Lu,Sh>` — declared on the prog `spawn.wat`'s ThreadOpts `launch` spawns | **`apply`** |
| process | `Peer<S,R>` — `:wat::program::self-peer` | **`apply`** |
| the only static site | a hand-driver, `probe_arc209_c2_defservice_dispatch.wat:76` | direct call |

So `serve`'s declared `self` type is enforced in production by **nothing** — only by that fixture
and by `serve`'s own self-recursion. The generic `Locus/launch` impl *cannot* name a per-service
`serve` statically, which is why the `apply` exists; that part is honest genericity, not laziness.

## The open question (NOT ruled here)

**Should the safe edge be stated once — `Peer'<S,R> <: ThreadSelfPeer'<S,R>` — instead of
enumerated at each site?** If so, `serve` keeps its current `ThreadSelfPeer` annotation (the
permissive head), the thread tier still matches exactly, and the process tier's `Peer` becomes
**statically passable** — which is what unblocks a static call and a rootable closure walk. Every
remote locus, being wire, hands over a `Peer` and passes for free.

Two hard requirements on any such change, and they are the whole risk:

1. **ONE-WAY ONLY.** The reverse edge must never be written. `check.rs:12986`'s refusal must keep
   firing, and a negative test must assert it — the same discipline that keeps
   `:wat::core::Value` honest (arc 278 R7: the top type is a fixed point *because* the second,
   looser rule was never added).
2. **Identical type args only.** Do not invent variance on `S`/`R` while adding the head edge.

Cross-refs: `BRIEF-293.W.2d-peer-purity.md` (the mint), `DESIGN-293.W-deep-wire-wall.md` (the
mobility wall), `docs/arc/2026/05/209-defservice/` + `wat/spawn.wat:336-523` (the Locus protocol),
`docs/arc/2026/06/278-rules-engine/SEAM.md` (the consumer that needs the static call).
