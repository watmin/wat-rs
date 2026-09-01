# DESIGN-STONE — the import door is a session's birth, and must be charged like one

> **Origin (2026-08-31).** Class A7 of `VIGILIA-2026-08-30-WORK-LIST.md`, the **last Class A row**,
> found by `circumspicere`. Driven here at HEAD `8a3ec39a1`, both halves.

## Why — half 1: the import is charged to nothing, and it is worse than "uncounted"

`grep 'check_session_ceiling\|mark_session_origin' src/rete/export.rs` → **no hit.** Neither call
exists at the import door.

`session_bytes(key)` (`alloc_counter.rs:243-251`) does `entry(key).or_insert(now)`. So for a session
whose origin was never marked, **the first ceiling check sets the origin to that moment** — every
byte the import allocated is retroactively free, and the session's ceiling begins after its network
already exists. Driven, same 2 MB of allocation, two origins:

```
marked at birth (what arm-session does)   sees   2097268 bytes
never marked   (what import does)         sees         0 bytes
```

This is A4's defect at a door A4 did not cover. A4 (`42704d57b`) fixed the thread-wide *clobber*;
this is the **never-marked** case, and the machinery A4 built is what makes the cure available.

## Why — half 2: the build is quadratic, and there is no cap on N

`export.rs:2128` builds the network through `PMap::from_pairs`, whose accumulator does
`acc.iter_mut().find(...)` per pair (`pmap.rs:150`) — a linear scan of everything already
accumulated. Driven, six samples per point, minimum taken:

| pairs | min | per-pair |
|---:|---:|---:|
| 500 | 523 µs | 1.05 µs |
| 1 000 | 1 954 µs | 1.95 µs |
| 2 000 | 5 143 µs | 2.57 µs |
| 4 000 | 19 473 µs | 4.87 µs |

**Per-pair cost doubles as N doubles** — the quadratic signature. Extrapolated on that curve, a
100 000-node Export costs on the order of **12 seconds** of CPU inside one import call, from bytes
some other process wrote, with the whole build charged to no session.

## ★ THE ONE CONTRACT DECISION

**The import door opens a session, so it marks the origin BEFORE it builds and refuses past a
stated node cap.** After this strike, what an import allocates is charged to the session it creates,
and how much it may allocate is a property of the declared format rather than of how long the caller
is willing to wait.

## ⚠ THE ORDERING TRAP — the origin cannot be keyed until the thing it keys exists

`mark_session_origin(key)` reads `thread_bytes()` **at call time**, and the key is the network's
`rust_identity`, which does not exist until the network `PMap` is built. So:

- mark *after* the build → the build is excluded, which is the defect;
- mark *before* the build → there is no key yet.

**The cure is to capture the origin before and file it after:** read `thread_bytes()` first, build,
then record *that captured value* under the new network's identity. That needs a sibling of
`mark_session_origin` taking the origin explicitly. It must keep A4's non-clobber rule — an origin
already filed for that identity wins, for the reason A4's closure gives.

## The node cap, and why it is MEASURED

A6's depth wall is the precedent at this same door: a stated constant, refusing with `malformed`,
with the measurement written at the constant. Same here — **measure the largest node count the
corpus actually produces**, then state the cap with the arithmetic above beside it, so a reader can
see what the cap costs in the worst case. A cap of 10 000 costs ~122 ms on the measured curve; a cap
of 50 000 costs ~3 s. That trade is the constant's whole justification and must be written down.

## Blast radius

`src/rete/export.rs` (both calls + the cap), `src/alloc_counter.rs` (the explicit-origin sibling),
and probes. **`pmap.rs` is NOT in the radius** — see the cut below.

## Out of scope — AFFIRMATIVELY CUT

- **⛔ Making `from_pairs` linear.** Tempting, and it is the row's second clause, but it is a
  *performance* change to a type the whole runtime uses, measured at 1.05–4.87 µs/pair, and its
  blast radius is every `PMap` in the tree. **The cap is what makes the quadratic safe**: bound N
  and the worst case is bounded with it. Once this strike lands, a linear `from_pairs` is a pure
  speed stone with a recorded before-curve to beat — and it is strictly easier to justify then,
  because the correctness argument no longer rests on it. **Do not touch `pmap.rs` here.**
- **Surfacing a ceiling breach at import as a new outcome variant.** The other five walls at this
  door refuse with `malformed`; a sixth that invents an outcome shape is a wire-visible change
  behind this arc's outcome wall. Refuse the way the neighbours refuse.
- **A4's per-session origin machinery itself.** Landed, closed, and this strike consumes it rather
  than revisiting it.
