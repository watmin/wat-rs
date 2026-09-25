# FINDING — what relies on the thread escape hatch (`ThreadSelfPeer`)

**Measured 2026-09-24/25** in an isolated worktree at `a830b3d8d`; nothing landed. The orchestrator
re-read both floor Summary lines and the probe log's payload distribution.

## The question and the ruling it serves

Builder: *"services must be loci agnostic … clients and servers may only pass data … not resources."*
Only EdnRepresentable may cross, and Rust already says so (`pub trait EdnRepresentable`,
`src/comms/mod.rs:102`, *"the wire contract, full stop"*). The wat checker has an escape hatch:
*"ThreadSelfPeer' is the escape hatch for thread workers that carry impure I/O … Any I/O is allowed
in-locus"* (`src/check.rs` ~:10945). **The builder did not know it existed.**

## What the hatch is: mostly a check nobody wrote

The §7 purity wall (`check_wire_peer_purity`, `check.rs` ~:10269) is applied **only where a `Peer` is
produced**: `self-peer`, `connect`, `accept`. The thread-spawn producer (`infer_thread_prog_type`),
`send`/`recv` (`project_peer_io`) and `poll` never call it. So the hatch is less an exemption than an
unwritten check.

## Results

Floors (worktree `.floor/`):

- the hatch closed statically: `6094 run: 6087 passed, 7 failed`. All 7 reds come from 3 deliberate
  fixtures.
- the hatch open, plus a runtime probe of every sent payload: `6094 run: 6093 passed, 1 failed`. The red
  is the lint firing on the probe's own source text, caused by the instrument.

A two-binary `--check` sweep over 2625 files: **exactly 3 change verdict**, the fixtures
`probe_arc293_W2{a,c,d}_*`.

| class | count | seen by | what |
|---|---|---|---|
| (a) a service's **client/server** Op/Reply channel carrying a resource | **0** | static + runtime | — |
| (b) a service's **lineage** channel | 1 code site (`wat/spawn.wat:513/520`, ThreadOpts `Locus/launch`), **35 services, 2461 sends** | **runtime only** | `<svc>::Status.Started {addr}` with a **Shared** `Address`, on the thread tier. The process tier ships a Wire capability |
| (c) a raw thread worker | 1 site (`wat/bracket.wat:735`), 3 sends | **runtime only** | `PoolMsg.Setup {deps}` with a **Shared** `Address` to a thread pool worker |
| (d) fixtures exercising the hatch | 3 files | static | intentional |

**No `Sender`, `Receiver`, fn or struct crossed any non-fixture channel on the floor.** Payload types
across 2594 flagged sends: `Status` 2548, `Admin` 18, `PoolMsg` 16, `Address` 9, test `S` 2, `Msg` 1.
**Zero Op/Reply**, on either tier.

## ⛔ A real hole found on the way: purity is blind through a generic type

`is_pure_type` checks only a parametric head's **type arguments**. It never substitutes them into the
declared fields. So a Shared `Address` reached through a Pure generic enum is judged pure:

- `(ThreadSelfPeer :- [(Address :- [i64 i64 Transport.Shared]) i64])` is refused with the hatch closed;
- **the same address inside `(E :- [Transport.Shared])` is accepted.** That is exactly defservice's
  `Status` shape, and it is why (b) and (c) are invisible statically.
  (`wat-scripts/scratch-pad/probe-tsp-hatch-generic-enum-blind.wat` in the worktree.)

**A purity wall that a generic wrapper defeats is not a wall.** This is its own stone, independent of
the rulings below.

## For the builder

1. **Is a `Shared` address data?** It is the in-process twin of a Wire capability that already crosses
   by design. Under *"only data, not resources"*, a crossbeam channel handle is a resource. The measured
   consequence of saying **no**:
   - thread-tier `Status.Started` (lineage) and `PoolMsg.Setup` (bracket) must carry something data-shaped;
   - the hatch can then close with only the 3 fixtures changing.
2. **Is the lineage channel (owner ↔ child supervision) under the client/server rule?** Measured: it is
   the **only** service channel carrying a resource.
