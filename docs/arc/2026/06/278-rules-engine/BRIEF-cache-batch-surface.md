# BRIEF — the `Cache<K,V>` surface goes BATCH, in both directions

> **A correction, not a feature.** `docs/CONVENTIONS.md:658` (arc 119) states a substrate law:
> *"Every wat-rs-shipped service exposes only batch-oriented `get`/`put` interfaces. Console is the
> single exception."* The `Cache<K,V>` surface shipped in `f4df1760` is **single-key**. That was my
> miss when I briefed Stone 2 — I designed the replacement for a compliant service without reading
> the convention it had to obey.
>
> **Builder-ruled 2026-07-26:** *"i'm /very/ ok with sending batch items in both directions…
> batch-write and batch-read are what i want… a user wanting to read exactly one item can just
> produce a vec of one item."*
>
> **The shapes below are SETTLED. Transcribe them; do not re-derive them.**

## Why this is not cosmetic

The convention's own reasoning names the defect exactly:

> *"Single-item interfaces **lie about the lock model** — they imply per-item acquisition when the
> loop already serializes."*
> *"The cache service **IS** a mutex implementation… **lock granularity = batch granularity**."*

An actor serializes one request at a time. With a single-key surface, a caller with N keys pays N
round trips and the actor holds its turn N times — and the caller has **no knob** to do better. The
batch size *is* that knob. The convention was written for the cache service; we built its
replacement without it.

Also: the oracle being deleted in Stone 5 **was compliant** — `Get(Vec<HolonAST>)` with
index-aligned results plus an empty-probe case. Without this, we delete a compliant service and its
proofs and replace them with a non-compliant one.

## The settled shapes

```clojure
;; ── get: batch in, INDEX-ALIGNED batch out ──────────────────────────────────────
(:wat::core::defrecord :wat::cache::Cache::GetRequest<K> [probes <- :wat::core::Vector<K>])

(:wat::core::defenum :wat::cache::Cache::GetResult<V> :wat::enum::Pure
  :Hit  [value <- :V]
  :Miss [])

(:wat::core::defenum :wat::cache::Cache::GetResponse<V> :wat::enum::Pure
  :Ok               [results <- :wat::core::Vector<wat::cache::Cache::GetResult<V>>]
  :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
  :RequestMalformed [path <- :wat::core::Vector<wat::core::String>
                     expected <- :wat::core::String  got <- :wat::core::String])

;; ── put: batch in, nothing meaningful out ───────────────────────────────────────
(:wat::core::defrecord :wat::cache::Cache::PutRequest<K,V>
  [entries <- :wat::core::Vector<wat::cache::Entry<K,V>>])   ;; Entry is Stone 1's — reuse it

(:wat::core::defenum :wat::cache::Cache::PutResponse :wat::enum::Pure
  :Ok               []
  :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
  :RequestMalformed [path <- :wat::core::Vector<wat::core::String>
                     expected <- :wat::core::String  got <- :wat::core::String])
```

**Verbs stay `get` / `put`** — the `Vector` in the signature already says batch.

Two places the convention itself is **stale**, and the settled shapes deliberately depart from it —
say so in the file's header so nobody "corrects" them back:

- Arc 119 says `get -> Vec<Option<V>>`. We use a **named** `GetResult<V>` enum, per the builder's
  later named-enum doctrine (*"a proper enum name is doubly useful"* — `Result/Ok` tells you
  nothing). It is also the extensible choice: a miss in a *similarity* cache has kinds (below the
  floor vs nothing stored), and `Option` forecloses ever saying which.
- Arc 119 says `put -> :wat::core::nil`. A bare `nil` is no longer expressible for a serviceable op
  — every op-Response must carry `RequestTooLarge` and `RequestMalformed`. `:Ok []` is what `nil`
  became; the shipped precedent is `:wat::kernel::StdOut::WriteResponse`.

`put` answering nothing also **resolves an honesty problem**: the current `Ok[displaced]` cannot be
told truthfully by both satisfiers (`HolographicLru::put` returns `nil` and never exposes the
evicted key, so `hologram-svc` would have to answer "nothing displaced" when things *were*).
Eviction reporting, if ever wanted, belongs where it can be honest — not in `put`'s reply.

## Read in order

1. **`wat/cache.wat`** — everything you touch is here: the `Cache<K,V>` surface, `lru-svc`,
   `hologram-svc`, and Stone 1's `Entry<K,V>` / `Lru::*` / `HolographicLru::*`.
   Both `:impls` become folds over the request vector. Note the asymmetry you are *removing*:
   `Lru::put` returns `Option<Entry>`, `HolographicLru::put` returns `nil` — with `:Ok []` neither
   needs to report it.
2. **`docs/CONVENTIONS.md:658-700`** — the convention, its reasoning, and the batch-of-one note.
3. **`wat-tests/service-cache-lru.wat`** and **`wat-tests/service-cache-hologram.wat`** — both gates
   move to the batch surface. Keep every behaviour they already prove; add index alignment.

## ★ The gate — INDEX ALIGNMENT is the load-bearing property

This is the one thing a batch API silently gets wrong while every single-key test still passes, and
it is precisely what the oracle's `hcs-helper-get-many-keys` proved before we deleted it.

In **both** gates, at minimum:

1. **Interleaved hits and misses, one round trip.** Store some keys; `get` a probe vector that mixes
   present and absent keys **in a deliberately jumbled order**; assert result *i* answers probe *i* —
   `Hit`/`Miss` in the right slots with the right values. A gate that probes all-hits or all-misses
   proves nothing about alignment.
2. **Batch put, then batch get** — several entries in one `put`, all readable in one `get`.
3. **Batch-of-one still works** — the builder's own argument for batch-only; prove the degenerate
   case is not degenerate.
4. **Empty probe vector** → `:Ok` with an empty results vector, not an error. (The oracle had this
   case; keep it alive.)

Everything the two gates already prove must stay green — multi-client, similarity across the wire on
both loci, eviction through the actor.

## Watch the budget

Both ops declare `:max-request-bytes 1024`. A batch request is *bigger than a single-key one by
construction*, and a `HolonAST` key is not small. **Ground what a realistic batch actually measures**
(`(string::length (edn::write req))` is what the guard measures) and raise the declared caps to fit,
with the number in the file's header saying what it was sized for. If a gate's own batch trips
`RequestTooLarge`, that is the cap being wrong, not the gate.

## STOP triggers — rejection criteria; report and ship nothing further

1. **If index alignment cannot be expressed** — if a fold cannot preserve probe order into results —
   STOP and report. Do NOT ship a batch API whose alignment is unproven; that is the one bug this
   shape can hide.
2. **If a realistic batch cannot fit under any sane `:max-request-bytes`** — STOP and report the
   measured size. That is a real finding about the fragmentation tooling, not something to paper
   over by shrinking the gate.
3. **If the blast radius exceeds `wat/cache.wat` + the two gates** — STOP and report before spending
   it. No `src/` Rust is expected.

## Method

- `target/release/wat --check <f.wat>` is your gate; read its printed output, never `$?` piped.
- **Load-order gate** — after any `wat/cache.wat` change, run this two-line program with the built
  binary and require `[]`:
  ```clojure
  (:wat::core::defn :user::main [] -> :wat::core::nil
    (:wat::kernel::println (:wat::deporder::verify-stdlib)))
  ```
- **You MAY run `cargo build --release`** — `wat/cache.wat` is baked via `include_str!`.
- **Do NOT run `cargo nextest`.** The orchestrator measures the floor centrally.
- Do NOT touch `crates/wat-lru/` or `crates/wat-holon-lru/` — live, and Stone 5's job.
- Foreground only. Scratch `.wat` → `wat-scripts/scratch-pad/`, green or deleted. Do not commit.

## Your report

The diff shape; the **interleaved-alignment** assertion quoted from a real run; the measured batch
size and the caps you chose; confirmation every pre-existing gate behaviour is still green on both
loci; `verify-stdlib` returning `[]`; any STOP.
