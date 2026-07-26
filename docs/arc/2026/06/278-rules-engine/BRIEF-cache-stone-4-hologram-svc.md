# BRIEF — cache Stone 4: `:wat::cache::hologram-svc`, the similarity cache as a service

> **The last build of the cache campaign.** Stones 1–3 shipped the two flavours and the service over
> the exact-key one; this puts `HolographicLru` behind the SAME surface, and then the crates can die.
>
> ```
> ✓ Lru<K,V>          ✓ Cache<K,V> surface   ✓ lru-svc<K,V>   ✓ HolographicLru   ✓ Match
> → hologram-svc      ← this stone
>   annihilate crates/wat-lru + crates/wat-holon-lru
> ```
>
> **Name RULED** (builder, 2026-07-25): `:wat::cache::hologram-svc`, mirroring `lru-svc`.

## What ships

One surface, two satisfiers — which is exactly why the surface got the plain name `Cache`:

```clojure
(:wat::service::defservice :wat::cache::hologram-svc
  :satisfies  :wat::cache::Cache<wat::holon::HolonAST,wat::holon::HolonAST>
  :durable    [ …a PURE seed — see crux 2… ]
  :ephemeral  [cache <- :wat::cache::HolographicLru]
  :init …  :impls [(get …) (put …)])
```

`HolographicLru` is concrete over `HolonAST` (the Hologram store is HolonAST-keyed), so both `K`
and `V` pin to `:wat::holon::HolonAST`.

## ★ CRUX 1 — a concrete service satisfying a PARAMETRIC surface is unprecedented

Grounded this session: the **only** `:satisfies` with type arguments anywhere in the corpus is
`wat/cache.wat:132`, `lru-svc<K,V>` satisfying `Cache<K,V>` — parametric satisfying parametric. A
**non-parametric** service satisfying a parametric surface at **fixed** arguments has never been
done here.

The parametric-protocol work (`1ac85d96`, `69d7dd5a`) exists precisely so a surface's type params
reach the wire, so this *should* work — the machinery is there. But it is untested, and the
`defservice` macro derives names from the `:satisfies` clause by splitting base from type-args
(`proto-base` / `proto-tp`, `wat/service.wat:268-276`). Whether that split behaves when the args are
concrete FQDNs rather than bare binders is exactly the untested part.

**Ground it FIRST, before writing the service.** A ten-line probe: a trivially small concrete
service satisfying an existing parametric surface at concrete args, `--check`ed. If it type-checks,
proceed. If it does not, **STOP** — that is a substrate finding and the builder's call, not
something to route around by making `hologram-svc` artificially generic.

## ★ CRUX 2 — the filter is a live closure, so it cannot be an init argument

`:wat::holon::filter-coincident` (`wat/holon.wat:65`) returns
`Fn(f64)->bool` — and it is a **closure that captures ambient state**: it reads
`:wat::config::dim-count` at call time and captures the resulting floor. `HologramLru::new` takes
that filter plus a capacity.

`Admin::Init` is **unconditionally Pure**, so an impure init argument is uncompilable (293.W). You
almost certainly cannot hand the service a filter value.

**The shape that works is already precedented** — the stdio-as-defservice stones hit exactly this
wall and solved it: `:durable` holds a **pure seed**, and the live thing is **born inside `:init`**
from that seed (there, an fd *number* → a live handle via the whitelisted `from-fd`). Mirror it:

```clojure
:durable   [capacity <- :wat::core::i64
            filter <- <a PURE seed naming WHICH floor — a keyword or a small enum>]
:ephemeral [cache <- :wat::cache::HolographicLru]
;; :init  → map the seed to its filter constructor, then HolographicLru::new
```

Choose the seed's type by the four questions and say which you chose and why. There are three
filters (`filter-coincident` / `filter-present` / `filter-accept-any`, `wat/holon.wat:65-93`) — a
closed set, which is a strong hint about the honest shape. **If a `Fn`-typed durable or init arg
turns out to be legal, STOP and report that instead** — it would contradict the purity wall and
that is worth knowing.

## Read in order

1. **`wat/cache.wat`** — `lru-svc` is your exemplar; copy its shape. Note it holds its handle in
   `:ephemeral`, born in `:init` from the durable capacity, and returns `s` unchanged from both
   impls because mutation happens inside the opaque handle. `HolographicLru::get`/`::put`/`::len`
   are the verbs you are wrapping (note `put` returns `nil`, unlike `Lru::put`).
2. **`wat/holon.wat:65-93`** — the three filters and what each floor means.
3. **`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`** — the ORACLE's service
   tests, and the reason this matters: those ~19 tests are the coverage that dies with the crate.
   **Read them for BEHAVIOURS worth gating; do not copy their forms** (hand-rolled channels,
   pair-by-index, the retired `LocalCache`). If one of them tests something your gate does not,
   that is the useful find — say so in your report even if you do not build it.
4. **`wat-tests/service-cache-lru.wat`** — the Stone 2 gate; your gate is its sibling. Steal the
   dial idiom (the separately-typed `…/dial` verb is load-bearing — it is where the type args are
   pinned).

## The gate — a `deftest`, BOTH loci

1. **Multi-client** — one service, two clients off `Handle/addr`; A `put`s, B `get`s and sees it.
2. **★ Similarity ACROSS THE WIRE** — B probes with a **different but coincident** `HolonAST` and
   hits. This is the one that matters: it proves the similarity match survives encode → wire →
   decode, which no in-process test can show.
3. **Miss is a value** — a non-coincident probe returns the `Miss` variant, not an error.
4. **Eviction is visible through the service** — at a small capacity, an overflowing `put` and a
   later `get` of the evicted key shows the dual-eviction invariant holding *through the actor*.

Assert on structure exactly; never a `contains` on a rendered string.

## STOP triggers — rejection criteria; report and ship nothing further

1. **If the concrete-satisfies-parametric probe fails** (crux 1) — STOP and report the diagnostic
   verbatim. Do NOT make `hologram-svc` parametric to dodge it; `HolographicLru` is concrete and a
   fake `<K,V>` would be a lie in the type system.
2. **If the filter cannot be reconstructed in `:init` from a pure seed** (crux 2) — STOP and report
   what the wall actually says.
3. **If the blast radius exceeds `wat/cache.wat` + the new gate** — STOP and report. No `src/` Rust.

## Method

- **`target/release/wat --check <f.wat>` is your gate** — it is healthy again as of `fdc2135c` (the
  `Hologram/find'` prime restored it). Read its printed output, never `$?` through a pipe.
- **You MAY run `cargo build --release`** — `wat/cache.wat` is baked via `include_str!`, so wat-side
  edits are invisible to `--check` until you rebuild.
- **Do NOT run `cargo nextest`.** The orchestrator measures the floor centrally, once.
- **Do NOT touch `crates/wat-holon-lru/` or `crates/wat-lru/`.** They are live (workspace members,
  `wat-cli` dependencies, ~19 tests in the floor) and their annihilation is Stone 5.
- Run everything in the FOREGROUND to completion. Do not background a command and return.
- Scratch `.wat` → `wat-scripts/scratch-pad/`, loader-gated: green, or deleted.
- Do not commit.

## Your report

The crux-1 probe result quoted verbatim; the durable seed's type and the four-questions reason for
it; the diff shape; the four gate behaviours from a real run — **especially #2, similarity across
the wire**; anything the oracle's service tests cover that your gate does not; any STOP. No
test-suite numbers — those are the orchestrator's.
