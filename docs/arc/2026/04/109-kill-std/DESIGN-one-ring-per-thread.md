# DESIGN — one ring per thread, lazily created

**Drawn 2026-09-16**, builder-directed (*"draw it"*), implementing the ruling in
`NOTE-the-reactor-is-used-as-a-disposable-poller.md`. **NOT STRUCK.**

Read that NOTE first: it carries the measurement that forced this (1024-ring per-UID ceiling,
proven by 332+254+254+184 = 1024 across four concurrent processes on the CI runner), the ruling
verbatim, and the wat-surface freeze this DESIGN turns into gate rows.

## The ruling, restated as the shape to build

1. **Exactly one `IoUring` per thread.** Not per `Receiver`, per `Sender`, per timer, or per call.
2. **Lazily created** — a thread that never performs process-tier IO allocates none. The **thread
   tier allocates none at all** (crossbeam, futex-based) and must continue to.
3. **Every operation goes through the waiting thread's ring**, demultiplexed by `user_data`.

⭐ **Ring count must track THREADS (bounded, small), never WAITERS (unbounded).** That sentence is
the whole stone; everything below is how to get there without moving the wat surface.

## ⭑ HALF OF IT EXISTS — this is a RE-SCOPING

Stone E-2 already built the mechanism, under the name this DESIGN will reuse:

```rust
/// Lazy persistent ring + its capacity, as a single noun.
type RingSlot = Option<(IoUring, u32)>;                     // src/comms/process.rs:236
```

Lazy (`None` until first use), persistent (survives calls), and **autoscaling** — capacity is
`next_power_of_two(arm_count).max(2)`, compared against the stored value at every entry and rebuilt
only when the structural need changes (`:1820`–`:1835`, `:2048`–`:2057`). The source calls it *"the
reflexive rebuild discipline."*

**The gap is ownership.** `RingSlot` belongs to `Select<'a, T>`; `Receiver` still **eagerly** owns
`ring: RefCell<IoUring>` at 4 entries apiece (`:315`, built at `:1104`). That is the per-waiter cost.

### The measured surface

```
Receiver's own ring borrows   9   :655 :788 :808 :980 :1027 :1033 :1100 :1168
Select's RingSlot borrows     6   :1715 :1829 :1847 :2051 :2064
Receiver construction sites   4
IoUring::new sites            9   ALL already funnelled through one `new_ring()` helper
```

`new_ring()` (added 2026-09-16 with the ring census) is the chokepoint: routing it at a thread-local
is the mechanical half of the change.

## ⛔⛔ THE INVARIANT — THE WAT SURFACE IS FROZEN

**No wat program may be able to tell**, by behaviour, types, timing class, or error text:

- `:wat::kernel::after` still returns a **`Peer`**; a timer stays selectable beside other peers with
  the same tier rules (`peer-wire?`).
- `select` keeps its fan-in, `ServiceEvent` arms, **index semantics** (idx 0 peer / idx 1 timer,
  which `call-by-deadline` depends on) and its mixed-tier refusal.
- `RecvOutcome` / `SendOutcome` / `TrySendOutcome` / `ConnectOutcome` / `CloseOutcome`: **no variant
  added, removed, or re-meaninged.**
- **No new wat verb** may be introduced to ease the substrate's job.
- Generated `defservice` code untouched: `child-main`, the serve loop, client methods,
  `call-by-deadline`, `race-reply`.

⭑ **If a `wat/` file changes, the substrate has leaked into the surface.** Justify it or revert it.

⚠ The one wat-visible failure that MAY deserve to change — `after` raising `MalformedForm` when a
ring cannot be created — is the **third missing-form sibling** and **a separate ruling**. Do not
smuggle it in here.

## ⭐ THE ACCEPTANCE TEST

**Delete the `ulimit -l` stopgap from `.github/workflows/ci.yml` and CI stays green.** The stopgap
exists only because ring count tracks waiters. When it tracks threads, the 8 MB / ~8 KB = ~1024
per-UID budget stops being reachable by a 4-way-parallel floor. Anything less is headroom, not a fix.

## Trap-doors — named so they are designed, not discovered

1. ⛔⛔ **THE NESTED BORROW IS THE DESIGN PROBLEM.** `Select` borrows its ring, then calls `Receiver`
   methods that borrow theirs; the code keeps them in **different `RefCell`s deliberately** —
   *"Select-ring borrow released; safe to call Receiver methods below (Receiver borrows its own ring;
   different RefCell)"*. One thread-local slot makes that collide on day one. Design the borrow
   discipline first (release-before-call, or pass the slot down explicitly); do not discover it.
2. ⛔ **The ring's fd is part of the Receiver's identity.** `:788` and `:1100` return
   `vec![data_fd, self.ring.borrow().as_raw_fd()]`, and the module header says *"owns its data fd AND
   its io_uring ring fd; both must survive."* Those lists must name the **thread's** ring now.
3. ⛔ **Fork.** `spawn.rs` forks. A child must **rebuild** its thread-local ring, never reuse one
   created pre-fork. `runtime.rs`'s `SHUTDOWN_SIGNAL_FD` rebuild is the precedent to copy.
4. **`Receiver` is `Send` by derivation** (no explicit `unsafe impl`). Establish whether one is ever
   created on one thread and waited on by another — the ring must be the **waiting** thread's. If
   they do cross, a thread-local *lookup* is right and a thread-local *owner field* is wrong.
5. **`AsyncCancel` must target the right `user_data`** and must not disturb another waiter's
   operations now sharing the ring.
6. **Capacity sizing generalises, it does not disappear.** `needed_capacity` must cover the widest
   submission live on that thread (largest select fan-in plus its timer), not the peer count.

## Out of scope — REJECTED for this stone

- **`ATTACH_WQ`, registered files, SQPOLL, real batching.** How the pattern is finished at scale;
  each is its own stone and none is needed to satisfy the ruling.
- **One ring per PROCESS.** Requires a poller thread and an event loop; the blocking
  `submit_and_wait(1)` model is compatible with per-thread and *not* with per-process. Do not drift.
- **Replacing io_uring with `ppoll`/`epoll`.** Ruled out by the builder: *"we are not undoing
  anything."*
- **Making `after`'s ring-refusal faceable.** Separate ruling (see the invariant above).
