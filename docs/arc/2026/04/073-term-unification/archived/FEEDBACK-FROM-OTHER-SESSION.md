# Arc 073 — Feedback from the consulting session

**Date:** 2026-04-28.
**Source:** the consulting session the user invited per `BUILD-LOG.md`. This session arrived from the consumer side: working through the lab cache architecture in conversation, landing on `HolonStore<V>` as the consumer-facing shape, and only then reading the BUILD-LOG to discover infra had been building the same architecture from the substrate side.

**What this session has read:**
- `BUILD-LOG.md` (the cold-start brief)
- `DESIGN.md` (slices 1–3 + slice 4 spec, including the values-up vs three-line-shim inconsistency)
- The relevant BOOK chapters (Ch 61 *Adjacent Infinities*, Ch 62 *The Axiomatic Surface*, Ch 65 *The Hologram of a Form*, Ch 66 *The Fuzziness*, Ch 70 *Jesus Built My Hotrod*, Ch 71 *Vicarious*)
- The lab cache umbrella `holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DESIGN.md`
- Proof 018 (`wat-tests-integ/experiment/022-fuzzy-on-both-stores/`)
- Proof 019 (`wat-tests-integ/experiment/023-population-cache/`) — this session's prototype of the cosine-readout pattern

---

## Recommendation: B — thread-owned mutable

The active session's Option B is the right call. Reasoning below.

### 1 — Hot-path puts make A expensive

The trader writes *both* next-form and terminal at every walker step. Per proof 018's pattern (and 059-001's chain-walking pattern):

```
trader's main loop, per candle:
  walk thought through eval-step!
  for each coordinate visited:
    record (form → next-form) in next-cache.put
    record (form → terminal) in terminal-cache.put
```

At 651,608 candles (the substrate's `data/analysis.db` corpus) over ~40 minutes, that's hundreds of thousands of puts. Under A, every put clones a `HashMap` — Arc bumps unmodified buckets and `Arc::make_mut`-style copy-on-write for the affected one. Cheap for an occasional update; the trader's pattern is *not* occasional.

The DESIGN's Option A summary already names this cost (*"questionable for a hot cache that hits put on every call site that produced a coordinate"*) but doesn't weigh it against the trader's actual workload. The workload tips it: the trader is THE first consumer; its hot path defines the substrate's performance contract for this primitive.

### 2 — LruCache is the precedent

Wat-rs already ships `:rust::lru::LruCache` via `crates/wat-lru/src/shim.rs` with `#[wat_dispatch(scope = "thread_owned")]`. The trader's existing encode-cache consumes it; the entire lab-cache umbrella was conceived around that shape. TermStore mirroring it keeps the cache surface coherent — one substrate pattern, one mental model.

If TermStore is the *only* substrate cache type that's values-up immutable while every adjacent cache type (`LruCache`, future `EngramLibrary`, et cetera) is thread-owned mutable, that asymmetry surfaces as friction at every consumer boundary. Consistency wins.

### 3 — values-up is the boundary discipline, not the within-thread rule

`ZERO-MUTEX.md` establishes the three tiers explicitly:

| Tier | Mechanism | Used for |
|---|---|---|
| 1 — Immutable | `Arc<T>`, frozen at startup | Config, symbol table |
| 2 — Thread-owned | `ThreadOwnedCell<T>` | Per-thread hot state |
| 3 — Program-owned | Spawned wat program + channels | Cross-thread shared state |

A thread-owned mutable `TermStore<V>` is **Tier 2 by construction.** It honors the substrate's no-Mutex discipline; the thread-id guard ensures scope safety; values-up flows at the BOUNDARY (Tier 3 service programs copy data via queues). The DESIGN's "values-up" framing is right at the boundary; it's the wrong mode within a thread's loop.

Within a thinker's tail-recursive loop, owning a mutable TermStore is *the same shape* as owning a mutable LruCache. The substrate already says this is fine.

### 4 — The three-line lab shim is real under B; aspirational under A

Under B:

```scheme
(:wat::core::define
  (:trading::cache::make-pair -> :(wat::holon::TermStore<HolonAST>,wat::holon::TermStore<HolonAST>))
  (:wat::core::tuple
    (:wat::holon::TermStore::new :None)        ;; next-cache
    (:wat::holon::TermStore::new :None)))      ;; terminal-cache
```

Three caches, one primitive, no glue. The recognition Chapter 70 named ("the substrate already KNEW this") becomes operational.

Under A:

```scheme
(:wat::core::define
  (:trading::cache::make-pair -> :(... mutable wrapper ..., ... mutable wrapper ...))
  (:wat::core::tuple
    (:trading::cache::make-mutable-termstore-wrapper ...)        ;; bespoke
    (:trading::cache::make-mutable-termstore-wrapper ...)))      ;; bespoke
```

Lab writes per-cache wrappers around the immutable TermStore. Each wrapper extracts template / slots / ranges, applies tolerance check, manages eviction, mediates put/get against a thread-owned mutable cell. *That's the substrate's job, not the lab's.* Three caches, three (similar but distinct) wrappers.

The DESIGN explicitly recognizes this in the Option A cons: *"Defeats the 'one primitive, three caches' framing."* It does. That framing is the architecture's whole point.

---

## Answers to the three guiding questions

### Q — Does the "three-line lab shim" framing still hold under your option?

**Under B: yes — literally.** TermStore<V> directly. No glue.

**Under A: no.** Lab writes a `:wat::lru::LocalCache<Template, Vec<(Slots, V)>>` wrapper per cache, with bespoke template-extraction + tolerance-matching + FIFO-eviction logic. The "one primitive, three caches" framing dies; lab-side cache code grows to mirror what the substrate already knows.

### Q — If A, what's the lab-side wrapper shape?

A thread-owned mutable cell carrying a `TermStore<V>`, with a put method that:

1. Calls `:wat::holon::term::template` on the input form
2. Looks up the existing bucket (if any)
3. Calls `:wat::holon::term::slots` to extract the slot vector
4. Applies tolerance check (`:wat::holon::term::matches?` or hand-rolled equivalent)
5. Decides FIFO position
6. Constructs the new TermStore<V> via `TermStore::put` (immutable)
7. Updates the mutable cell to hold the new TermStore<V>

Performance-wise, this is dominated by step 6 (the immutable put's clone cost). Steps 1-5 are the substrate's already-shipped term primitives, just called from lab. Step 7 is the mutable-cell write.

The throughput hit comes from step 6: cloning the HashMap on every cache write. The lab umbrella's performance contract is **≥272 candles/sec sustained on 10k**. That's tight; per-call HashMap clones probably blow that budget. (Would need to benchmark to confirm exactly, but the active session's "questionable" intuition matches.)

### Q — If B, are there cross-thread / cross-program coordination cases the active session is missing?

None I can identify. The standard lab cache stratification:

- **L1 (per-thinker, hot path):** `TermStore<V>` directly, thread-owned. Mirrors LruCache today. ✓
- **L2 (cross-thinker, single process):** service program owns a `TermStore<V>`; thinkers `send`/`recv` put/get requests via queues. Mirrors `:wat::lru::CacheService`'s shape. The TermStore lives in ONE thread (the service program's loop); the values-up discipline applies AT the queue boundary. ✓
- **L3 (cross-process, durable):** a *different impl* with the same put/get/len signature — backed by SQLite or similar. Slice 4 doesn't ship this; future arc when persistence surfaces. The `architecture-is-interface` rule (this session's memory: feedback_architecture_not_implementation.md) says: same call sites, different backing. ✓
- **The spell (BOOK Ch 67/71, networked):** yet another impl with the same signature, federated across machines. Future. Same architecture; different storage. ✓

`:rust::lru::LruCache` being thread-owned has not been an obstacle for the lab; TermStore being thread-owned won't be either.

### Q — A third option?

Worth flagging, not recommending:

**Hybrid: mutable `TermStore<V>` + immutable `TermSnapshot<V>` (produced by `freeze`).** Useful if a consumer wants to ship a value-up read-only handle across threads (e.g., publish a frozen population for analysis while the live store keeps mutating). Overkill for slice 4. If a Phase-2 consumer surfaces this need, future arc.

The simpler shape (B alone) handles every immediate use case. Adding `freeze` is feature creep until a consumer asks for it.

---

## Cross-session convergence note

The consulting session arrived at `HolonStore<V>` as a trait with three impls (`HolonHash` unbounded, `HolonCache` bounded with eviction, `HolonDatabase` durable) — coming from the consumer side, sketching what the lab needs. The active session shipped `TermStore<V>` as one concrete type with `sqrt(d)` cap baked in — coming from the substrate side, exposing what the algebra makes possible.

**Both are right at different layers:**

- `TermStore<V>` is the **substrate primitive** — one shape, decomposition baked in, sqrt(d) invariant enforced. *This is what slice 4 ships.*
- The "Hash / Cache / Database" stratification is the **consumer-side organization** — same TermStore signature, different backings as more impls arrive over time. *This emerges naturally.*

**There is no need to design a trait abstraction at the substrate layer right now.** Slice 4 ships TermStore<V> as a single concrete in-memory mutable primitive. When future arcs add:

- `TermStore` backed by SQLite (BOOK Ch 71's L3 / vicarious framing)
- `TermStore` backed by a federated network (Ch 67's spell)
- `TermStore` backed by an engram library or external knowledge base

…each ships with the **same** put / get / len signature. Consumers using `TermStore<V>` swap the constructor (`TermStore::new` vs `TermStore::open(path)` vs etc.) without changing call sites. The trait abstraction the consulting session imagined is implicit in the signature; explicit trait machinery would be premature.

**Slice 4 ships as the substrate primitive; the consulting session's organizational framing is a future-arc concern that doesn't gate this slice.**

---

## DESIGN.md update

Three edits to make:

1. **Line ~119** — Strike `;; values-up — returns new store` from the `TermStore::put` signature comment. Replace with `;; thread-owned mutable; mutates in place. Returns Self for chaining ergonomics, but the same store is mutated.` (Or have `put` return `:()` if mutation-only is preferred; depends on the wat-rs convention.)

2. **Line ~131** — Strike the paragraph: *"TermStore is a value-up immutable structure (returns new store on `put`); the lab cache slice that needs thread-owned mutable state composes it inside a `LocalCache<Template, …>` or service program of its own choosing — *not* this arc's concern."*

   Replace with: *"TermStore<V> is a thread-owned mutable structure, mirroring `:rust::lru::LruCache`'s `#[wat_dispatch(scope = "thread_owned")]` shape. `put` mutates in place. Within a thread, owned mutable cells are Tier 2 per `ZERO-MUTEX.md` and honor the substrate's no-Mutex discipline. The lab cache slice consumes `TermStore<V>` directly (the three-line shim is real). Cross-thread sharing routes through service programs (Tier 3); the values-up discipline applies at the queue boundary, not within a thread's loop."*

3. **Line ~180** — Strike *"This arc ships values-up (`put` returns a new store). The lab cache slice composes a thread-owned mutable cell around it if it needs that."*

   Replace with: *"This arc ships thread-owned mutable. Future arcs add freeze/snapshot semantics if a consumer surfaces the need; not in slice 4."*

Slice 4 is ready to ship as B with these updates.

---

## One technical note on the slice 4 implementation

For the LruCache mirror, the dispatch likely lands as:

```rust
#[wat_dispatch(path = ":wat::holon::TermStore", scope = "thread_owned", type_params = "V")]
pub struct WatTermStore {
    cap_per_bucket: usize,
    buckets: HashMap<HolonAST, Vec<(Vec<f64>, Value)>>,
    // FIFO eviction order is implicit in Vec append + drain-from-front
}
```

The `V` type parameter via `type_params = "V"` mirrors how arc 070 introduced `WalkStep<A>` (the first parametric built-in enum). If wat-rs's `#[wat_dispatch]` already supports parametric structs cleanly, this is straightforward. If not, that's the slice's first task — ship the parametric struct registration capability, then ship TermStore on top.

(The consulting session can't tell from this side whether parametric `#[wat_dispatch]` structs already work. If they don't, that's worth flagging to the user as a slice-4 sub-concern; if they do, this is a non-issue.)

---

## Summary

- **Slice 4 ships as B** (thread-owned mutable), mirroring `:rust::lru::LruCache`.
- **DESIGN.md needs three edits** (above) to remove the values-up framing.
- **The trait abstraction the consulting session arrived at is a future-arc concern**, not a slice-4 design decision. TermStore<V>'s signature is what makes future impls (SQLite, networked, engram-backed) compose without consumer changes.
- **No cross-thread coordination cases are missed** by going thread-owned.
- **Hybrid mutable+freeze is feature creep** for slice 4; defer until a consumer asks.

The lab umbrella 059 slice 1 unblocks immediately upon slice 4 landing. Proof 018's six tests become the substrate-tier regression suite per the DESIGN's existing test-strategy section. The three-line lab shim is the lab cache. Chapter 70's recognition operationalizes.

PERSEVERARE.
