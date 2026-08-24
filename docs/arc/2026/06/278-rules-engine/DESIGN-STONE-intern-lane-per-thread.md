# DESIGN-STONE — the map-intern counter is laned per thread, not one global atomic

> **Origin (2026-08-23).** Builder, on being told the last untouched
> perf axis was the allocator: *"we must have the rete subsystem be
> tolerant to highly concurrent execution — imagine 512 threads all
> running their own rete — they must never step on each other."*
> The audit that question forced found the hazard, and it was not the
> allocator.

## The enemy

```rust
fn next_intern() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
```

`src/value/pmap.rs`. One **process-global** cache line, taken exclusively on
every mint — and **every one-entry `PMap` mints**, which is **40 000 per fire**
on the harvest path alone (`harvest_wrap_parts`). N concurrently-firing retes
do not each pay their own cost; they serialise on this line.

This is the one place the engine's concurrency contract leaked. Everything
else already holds it: `rg Mutex src/rete` is empty, `ARM_TABLE` is
`thread_local` (stone 27, `DESIGN-STONE-intern-zero-mutex`), `ARM_BUILDS` is
`#[cfg(test)]`, and every other static in the fire path is a write-once
`OnceLock`. The audit is in the Weigh below.

## The measurement that convicted it

`intern_counter_thread_scaling` — 400k mints per thread on an 8-core box,
against a private twin counter so other tests cannot perturb it:

| threads | A: shared `AtomicU64` | B: per-thread lane |
|---:|---:|---:|
| 1 | 16.03 ns/op | 5.30 ns/op |
| 2 | 22.75 ns/op | 2.59 ns/op |
| 4 | 17.83 ns/op | 0.89 ns/op |
| 8 | **15.87 ns/op** | **0.47 ns/op** |

**A never improves with threads. B scales near-linearly.** At 8 threads that
is **34×** aggregate throughput, and B is faster single-threaded too (5.30 vs
16.03) because a TLS `Cell` read beats an atomic RMW.

⚠ **Honest about the noise:** A's absolute value is unstable run to run — the
1-thread reading came in at 5.80, 9.06, 16.01, 16.03 ns/op across four runs.
No precise figure is claimed. What is stable across all four is the **shape**:
A's throughput never improves as threads are added, B's improves every time.
That shape is the finding; the absolute nanoseconds are not.

B is measured through the **real** `next_intern`, TLS lookup included. An
earlier draft timed a bare `local += 1` loop and read 0.04 ns/op — the
optimizer had collapsed it. That number was discarded, not reported.

## The algorithm

```
INTERN_LANE_BITS = 20        // 2^20 lanes  (minting threads)
INTERN_SEQ_BITS  = 44        // 2^44 ids per lane

fresh_intern_lane():                    // ONE atomic per THREAD, not per mint
    lane = LANE.fetch_add(1)            // LANE starts at 1
    assert lane < 2^20
    return lane << 44

thread_local NEXT_INTERN = fresh_intern_lane()

next_intern():
    id   = NEXT_INTERN.get()
    next = id + 1
    NEXT_INTERN.set(next & SEQ_MASK == 0 ? fresh_intern_lane() : next)
    return id
```

## ★ THE ONE CONTRACT DECISION

**Mint ids are partitioned by thread; uniqueness is preserved, not traded.**
The high bits name the minting thread and the low bits count within it, so two
threads cannot mint the same id. Ids stay `u64`, stay outside `Eq`/`Hash`, and
stay clone-stable — nothing observable about a `PMap` changes. Ordering across
threads was never a property anything relied on, and is now explicitly not one.

Lane 0 is never issued, so **no id is ever 0** — the runtime already rejected a
shared intern id of 0 for one-entry maps
(`DESIGN-STONE-harvest-wrap-parts` § Out of scope), and this closes that hole
by construction rather than by convention.

## The gate

1. `intern_counter_thread_scaling`: A flat-or-worse with threads, B improving.
2. `laned_intern_ids_are_unique_across_threads`: 8 threads × 50k mints, all
   distinct, none 0.
3. Rete cohort green incl. `spec_equals_native_on_every_where_family`.
4. Floor GREEN. Clippy `--release --workspace --all-targets -D warnings`.
5. Single-thread grid does not regress.

## Predicted win

Written before the measurement: **concurrency is the point, not throughput.**
Single-threaded the grid should move by roughly 40k × (16.0 − 5.3) ns ≈
**−0.4 ms** on fanout `[40000]`, which is at the edge of that cell's ±0.2 ms
noise — so a null single-thread grid is an acceptable outcome and NOT a
refutation. The claim this stone makes is about the shape of the scaling
curve, which the grid (single-threaded by construction) cannot see at all.

## Blast radius

`src/value/pmap.rs` only — `next_intern` plus the lane constants and the
thread-local. No `.wat`. No Session field. No rete source touched. `PMap`'s
public surface, `Eq`, `Hash`, and arm behaviour are unchanged.

## Out of scope = REJECTED

- A global allocator (mimalloc/jemalloc). Still untried, still the last
  unexplored axis — but it is a **separate** decision from this one, and
  bundling it here would confound two effects in one weigh.
- Sharing one intern id across one-entry maps — already rejected; overlay
  identity is per instance.
- Making the id smaller than `u64`, or folding the lane into `Eq`/`Hash`.
- Recycling lanes when a thread dies. Lanes are consumed monotonically; a
  process that mints from more than 2^20 distinct threads panics loudly rather
  than silently colliding two map identities. That bound is stated, not hidden.

## Sequencing

1. Audit shared mutable state in the fire path. (DONE — this is the only one.)
2. Probe the scaling. (DONE.)
3. Lane it. Prove uniqueness. Weigh. Stop.
4. Revert if uniqueness fails or the single-thread grid regresses beyond noise.

## Weigh (2026-08-23) — LANDED

Floor **GREEN** `.floor/2026-08-24T00-07-40Z` — **4936 passed**, 19 skipped,
274.610s, no ARM. Rete cohort **358/358** incl.
`spec_equals_native_on_every_where_family`. Clippy CI-identical
(`--release --workspace --all-targets -- -D warnings`) **silent**.

`laned_intern_ids_are_unique_across_threads`: 8 threads × 50 000 mints =
400 000 ids, **all distinct, none 0**. Uniqueness is proven, not assumed.

### The concurrency audit the builder's question forced

Every piece of shared mutable state reachable from a fire, checked against the
disk this session:

| site | kind | verdict |
|---|---|---|
| `rg Mutex\|RwLock src/rete` | — | **empty** — stone 27 holds |
| `ARM_TABLE` (`arm.rs:625`) | `thread_local` | per-thread by design |
| `ARM_BUILDS` (`arm.rs:632`) | `AtomicUsize` | `#[cfg(test)]` — not in the fire |
| `vocabulary.rs:1373`, `step_payload.rs:178,199`, `fire/mod.rs:1476,1481`, `expr_ir.rs:1325`, `export.rs:76` | `OnceLock` | write-once then read-only; no contention |
| census `thread_local!` ×7 | `thread_local` | `#[cfg(test)]`, per-thread |
| **`pmap.rs:30` `NEXT: AtomicU64`** | **global atomic** | **THE LEAK — this stone** |

One hazard in the whole fire path, and it was on the hottest allocation site
in the engine. 512 retes on 512 threads now share **nothing** on the mint
path.

### What the grid can and cannot say

The grid is single-threaded by construction, so it cannot see this stone's
actual claim. It is run only to prove no single-thread regression — and the
TLS path is *faster* than the atomic even on one thread (5.30 vs 16.03 ns/op),
so a small improvement or a null are both acceptable; only a regression
refutes.

### What this does NOT claim

- **Not that 512 threads were measured.** This box has 8 cores. What is
  measured is that the shared counter's aggregate throughput does not improve
  from 1→8 threads while the laned one improves every step. Extrapolating the
  *shape* to 512 is sound; extrapolating a *number* is not, and none is given.
  Cross-socket NUMA would make the shared case worse than measured here, not
  better.
- **Not that the engine is now proven concurrent.** This removes the one
  contended line found by the audit. It does not test 512 live sessions, and
  the arm intern still requires connection-thread affinity (stone 27) — a
  session that migrates threads still misses its arming thread's row. That
  bound is unchanged and remains named in `DESIGN-STONE-intern-zero-mutex`.
- **Not that the allocator question is answered.** It is untouched and remains
  the last unexplored axis, deliberately not bundled here.

## The fuzz that closes this stone's own gap (2026-08-23)

This stone's evidence was a scaling probe over a **counter in isolation**. It
said so, and that was a real hole: nothing in the suite ever fired more than one
engine at a time. The builder named the contract precisely —

> *"there must be no shared state — N concurrent rete instances must never
> commingle any state… we need concurrent readers to operate independently —
> they get scheduled however they get scheduled but they never clobber each
> other."*

`tests/rete/probe_arc278_concurrent_retes.{rs,wat}` is that gate, built on wat's
own first-party thread pool (`:wat::bracket::map (:wat::spawn::thread)`).

**48 workers, each a whole engine.** Every worker compiles its own rules, builds
its own network, seeds its own facts, fires its own session and reads its own
queries. Nothing is passed between workers; no session is shared. The only thing
they have in common is the process — which is exactly where a global hides.

**Two rule sets interleaved.** Even workers run a `:cc` 3-stratum chain, odd
workers a `:dd` 2-stratum one, so two distinct compiled networks are live on the
pool simultaneously and the per-thread arm intern (stone 27) is exercised with
both. Each witness carries a rule-set **tag** as well as its counts, because the
counts alone are identical between the sets — without the tag, a worker reading
another thread's arm would be invisible.

**Workers finish out of step.** Worker `i` seeds `100 + i` items, so tasks are
different sizes. A pool where every task is identical can keep workers in
lockstep and hide the very interleaving the gate exists to find.

Five assertions: the serial reference matches the analytic closure; concurrent
matches serial element for element; concurrent *also* matches the analytic
closure (so a shared systematic error cannot hide); both rule sets are provably
live; and the whole pool is re-run 8 times, because a race that needs a
particular interleaving will not show on one pass.

### Two things deliberately NOT in this gate

- **No timing.** Perf under concurrency is out of scope here. A duration
  assertion on a shared box is a flake generator, and every red on this gate
  must mean cross-thread damage and nothing else. A first draft did measure
  speedup, read 1.35× on 8 cores, and nearly reported "poor scaling" — the
  controls showed the number was dominated by per-call world freeze and pool
  spawn, not by the retes. It was cut rather than fixed: it was answering a
  question this gate is not asking.
- **No shared session.** A draft also tried one fired base with concurrent
  readers and copy-on-write overlays. That is the wrong model for this contract:
  instances must share *nothing*, so deliberately sharing a session tests
  something the design does not promise. Removed.

**What it does not prove:** that the pool is multi-threaded. A pool that
silently ran every task on one thread would make it pass vacuously. That belongs
to the bracket layer and is covered by `tests/kernel/probe_arc259_brackets_*`.
It is named in the probe's header rather than assumed, because a vacuous green
is worse than a red.
