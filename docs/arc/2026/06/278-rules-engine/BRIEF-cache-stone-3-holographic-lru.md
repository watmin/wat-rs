# BRIEF — cache Stone 3: `:wat::cache::HolographicLru`, the similarity-keyed composite

> **Stones 1 and 2 shipped** (`a86f521c` the `Lru<K,V>` primitive, `f4df1760` the `lru-svc<K,V>`
> service + the `Cache<K,V>` surface). Stone 3 is the **other cache flavour**: same eviction
> discipline, different key-matching — exact-key becomes *hologram similarity*.
>
> **Name RULED** (builder, 2026-07-25): `:wat::cache::HolographicLru`. It IS an LRU — the only
> difference from `Lru` is how a key matches — so it carries the shared `Lru` label plus the
> `Holographic` qualifier. Concrete over `HolonAST` (the Hologram store is HolonAST-keyed, not
> generic). Do not re-open it.
>
> **This stone is the composite ONLY.** The service over it (`hologram-svc`) is Stone 4.

## What it is — two structures, one cache, and the invariant that binds them

A `defstruct` composing a **Hologram** (the similarity index, which holds the values) and an
**`Lru`** (the recency/bound index, which holds *keys only*):

```clojure
(:wat::core::defstruct :wat::cache::HolographicLru
  [hologram <- :wat::holon::Hologram
   lru      <- :wat::cache::Lru<wat::holon::HolonAST,wat::core::nil>])
```

The `nil` value slot is the point and must be said plainly in the header: **the LRU is not storing
anything — it is the bound and the recency order.** The values live in the Hologram. The LRU exists
so the Hologram cannot grow without limit.

**THE LOAD-BEARING INVARIANT — dual eviction.** The two structures must never disagree about which
keys exist. When `put` overflows the LRU, the LRU hands back the displaced entry and that key must
be **removed from the Hologram too**. Miss this and the Hologram grows forever while the LRU
believes it is bounded — the cache silently stops being a cache. Your gate must prove it (below).

`defstruct`, not `defrecord`: both fields are live impure handles. That also means a
`HolographicLru` can only ever live in a service's `:ephemeral` — but that is Stone 4's problem,
not this one's.

## Read in order

1. **`crates/wat-holon-lru/wat/holon/lru/HologramCache.wat`** — the ORACLE. Read it whole; it is
   short and it already solved this. Study especially `put` (the eviction → `Hologram/remove`
   chain) and `get` (`Hologram/find` → bump the matched key in the LRU). **This is a study oracle,
   not a source to copy** — the names are the old `:wat::lru::LocalCache` ones and the crate is
   being annihilated. Rebuild it clean on the Stone 1 primitive, exactly as Stone 1 was rebuilt
   from its own oracle rather than lifted.
2. **`wat/cache.wat`** — where this lands, and Stone 1's `Lru::new`/`put`/`get`/`len` +
   `Entry<K,V>`. Note `Lru::put` returns `Option<Entry<K,V>>` — the displaced entry. **That return
   is what makes dual eviction possible**; it is the whole reason Stone 1 lifted the oracle's bare
   tuple into a named `Entry`.
3. **The `Hologram` verbs — core Rust builtins, registered at `src/check.rs:17815+`, evaluated at
   `src/runtime.rs:4832+`.** Ground each signature there rather than trusting this list:
   `Hologram/make (filter) -> Hologram` · `Hologram/put (h, key, val) -> ()` ·
   `Hologram/find (h, probe) -> Option<…>` (the matched key AND the value — this is what `get`
   needs) · `Hologram/remove (h, key) -> Option<…>` · `Hologram/len (h)`.
   The filters live in `wat/holon.wat:65-93`: `filter-coincident` / `filter-present` /
   `filter-accept-any`.

## The verbs (type-scoped, mirroring Stone 1)

`HologramLru::new` / `::put` / `::get` / `::len`, all scoped under the type. **The BARE
`:wat::cache::get` / `:wat::cache::put` names stay RESERVED** — a later stone makes them ONE
`defclause` over both flavours (`Lru | HolographicLru`), and wat forbids two `defn`s sharing an
FQDN. Taking them here would block that stone. (`wat/cache.wat`'s header already says this; keep it
true.)

## The gate — a `deftest`, and it must prove the invariant, not just the happy path

1. **Similarity, not equality.** Put under one key; `get` with a **different but coincident** probe
   and receive the value. If the gate only ever probes with the exact key it stored, it has tested
   an `Lru` with extra steps and proved nothing about this stone.
2. **★ DUAL EVICTION — the one that catches a real bug.** At capacity N, insert N+1 distinct keys,
   then probe for the evicted key and assert it is **gone from the HOLOGRAM** (a miss), not merely
   absent from the LRU. A naive implementation passes every other test and fails this one.
3. **`get` bumps recency.** Put A then B at capacity 2, `get` A, then put C — B is evicted, not A.
   This proves the `Hologram/find` hit feeds back into the LRU's ordering.
4. **`len` agrees with the bound** after the overflow.

Assert on the STRUCTURE exactly — extract fields and compare. Never a `contains` on a rendered
string.

## STOP triggers — rejection criteria; report and ship nothing further

1. **If `Hologram/find` does not return enough to bump the LRU** (i.e. you cannot recover the
   *matched key*, only the value) — STOP and report the real signature. `get`'s recency bump
   depends on it and the oracle claims it is there; if the oracle is stale, that is the finding.
2. **If dual eviction cannot be expressed** — if `Lru::put`'s displaced `Entry` does not give you
   the evicted key, or `Hologram/remove` cannot take it — STOP and report. Do NOT ship a composite
   whose two halves can drift; an unbounded Hologram wearing a bounded cache's name is a lie.
3. **If the blast radius exceeds `wat/cache.wat` + the new gate** — STOP and report before spending
   it. This stone touches no `src/` Rust and does not modify the oracle crate.

## Out of scope — rejected, not deferred

The `hologram-svc` defservice (Stone 4), the unifying `get`/`put` `defclause` over both flavours,
any migration or deletion of `crates/wat-holon-lru` (Stone 5), and metrics.

## Method

- **You MAY run `cargo build --release`** (~36s) — and for this stone you will need to.
  `wat/cache.wat` is baked into the binary via `include_str!`, so a stdlib edit is invisible to
  `--check` until you rebuild. Build, then `--check` your gate. This exception exists because a
  prior rider on Stone 2 could not verify its own work without it.
- **Do NOT run `cargo nextest`.** The orchestrator measures the floor centrally, once.
- `target/release/wat --check <f.wat>` after a build is your gate; read its printed output, never
  `$?` through a pipe.
- `macroexpand` when a macro's output confuses you — read what was EMITTED before theorising.
- Scratch `.wat` goes in `wat-scripts/scratch-pad/` and is loader-gated: green, or deleted.
- Do not commit.

## Your report

The diff shape; the four gate behaviours quoted from a real run — **especially #2, the dual
eviction**; confirmation the LRU's value slot is `nil` and the values live in the Hologram; any
STOP. No test-suite numbers; those are the orchestrator's.
