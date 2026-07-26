# BRIEF — cache Stone 2: `:wat::cache::lru-svc<K,V>`, the multi-client cache service

> **Stone 1 shipped the primitive** (`a86f521c` — `:wat::cache::Lru<K,V>`, thread-owned, zero mutex).
> Stone 2 is the MULTI-CLIENT form: a `defservice` whose actor serialization *is* the mutex, so N
> clients share one cache with no lock written anywhere.
>
> **The blockers are gone.** The parametric protocol reaches the wire (`1ac85d96`) and messages now
> spell only the params they use (`69d7dd5a`). This stone is the first consumer of both.
>
> **The surface name is RATIFIED**: `:wat::cache::Cache<K,V>` — intueri-cast, then weighed against
> our own precedent (`:wat::capability::Capability`, `:wat::stream::Stream` — a single-concept module
> whose principal type takes the module name). Do not re-litigate it.

## What ships

```clojure
(:wat::core::defsurface :wat::cache::Cache<K,V> :nature :wat::kernel::Peer'
  :messages [ …GetRequest<K> / GetResponse<V> / PutRequest<K,V> / PutResponse<K,V>… ]
  :features [(get …) (put …)])

(:wat::service::defservice :wat::cache::lru-svc<K,V>
  :satisfies  :wat::cache::Cache<K,V>
  :durable    [capacity <- :wat::core::i64]        ;; EDN. hibernatable. the spec.
  :ephemeral  [cache <- :wat::cache::Lru<K,V>]     ;; the live handle. born in :init.
  :init …  :impls [(get …) (put …)])
```

## The one contract decision — durable holds the SPEC, ephemeral holds the HANDLE

`:wat::cache::Lru` is a thread-owned Rust opaque (`scope = "thread_owned"` on the shim). It
**cannot** cross a wire or a hibernation boundary, and 293.W enforces exactly that: an impure
surface-typed field may live only in `:ephemeral`. So:

- `:durable [capacity <- :i64]` — plain EDN, the *spec* from which the resource is rebuilt.
- `:ephemeral [cache <- :wat::cache::Lru<K,V>]` — born inside `:init` by calling
  `:wat::cache::Lru::new` on the durable capacity.

This is R5 at the service layer — store the thunk, not the answer — and it is **correct by
construction**, not by discipline: writing it the other way does not compile. Say so in the file's
header; a reader should learn the rule from the form.

## Read in order

1. **`wat/cache.wat`** — Stone 1, whole file. It is short. `Lru::new` / `Lru::put` /
   `Lru::get` / `Lru::len` and `Entry<K,V>`. Note `put` returns
   `Option<Entry<K,V>>` — the DISPLACED entry (LRU eviction, or the previous binding on an
   overwrite). That return is the interesting half of this service and the gate must observe it.
   Note also the header's standing instruction: the BARE `:wat::cache::get`/`put` names are
   RESERVED for a later stone's `defclause` over both cache flavours — **this stone must not take
   them.** The client verbs the macro generates are `lru-svc/get` / `lru-svc/put`, which is fine.
2. **`wat/kernel/services/stdio-primes.wat:30-50`** — the live shape precedent: a `defsurface`
   (`StdOut`) immediately followed by its `defservice` (`stdout-svc`), with a resource born in
   `:init` and held in `:ephemeral`. Copy this shape.
3. **`wat-tests/service-parametric-messages.wat`** — the proof-of-shape for a `<K,V>` service on the
   wire, both loci, `K=String` and `V=i64`. Your gate is its sibling; steal its dial idiom
   (the separately-typed `…/dial` verb is load-bearing, not stylistic — it is where K and V are
   pinned, and inlining `connect'` leaves K open where the purity wall inspects it).
4. **`wat-tests/service-parametric-bare-messages.wat`** — the just-shipped bare-message form.
   Messages here spell only the params they USE: `GetRequest<K>` names K only, `GetResponse<V>`
   names V only. Do not re-attach params a message does not use — that rule is dead.

## The gate (a `deftest` sibling, BOTH loci)

The load-bearing behaviours, in one round trip per locus:

1. **Multi-client.** Stand up ONE service; `connect'` **two** clients off `Handle/addr`. Client A
   `put`s, client B `get`s and **sees A's value**. That is the whole point of the stone — one
   cache, N clients, no lock — and it is the arc-130 N-client case landing natively.
2. **Eviction is observable.** Capacity 2; put three distinct keys; assert the `put` that overflows
   returns `Ok[displaced = Some(Entry …)]` naming the evicted key, and a later `get` of that key is
   `Miss`.
3. **Miss is a value, not an error.** `get` of an absent key returns the `Miss` variant.

Assert on the STRUCTURE exactly — extract the fields and compare them; never a `contains` on a
rendered string.

## STOP triggers — rejection criteria; report and ship nothing further

1. **If the `Lru` handle cannot be held in `:ephemeral` and born in `:init`** — STOP and report the
   exact checker diagnostic. Do NOT move it to `:durable`, do NOT make it a wire type, do NOT
   wrap it to dodge the purity wall. That wall is the design.
2. **If two clients cannot share one service's cache** (each somehow gets its own) — STOP and
   report. That would mean the actor is not the mutex and the stone's premise is wrong.
3. **If the blast radius exceeds `wat/cache.wat` + the new gate** — STOP and report before
   spending it. In particular this stone touches no `src/` Rust.

## Known and deliberate — do not "fix" it

The wire enforces every CONCRETE field but does not enforce `K` itself (wat erases type params, so
inside `serve<K,V>` there is no `K` to check against — measured, and written into
`service-parametric-messages.wat`'s assertion). `K` is pinned statically at the caller. A guard that
carries the instantiation is a separate, deliberate decision the builder has not made. Build the
static-`K` form; do not invent enforcement here.

## Method

- `target/release/wat --check <f.wat>` is your gate: foreground, ~0.2s. Read its printed output,
  never `$?` through a pipe.
- **Do NOT run `cargo build` or `cargo nextest`.** The orchestrator measures the floor centrally,
  once, after your edits land. Running them here contends on the one `target/` lock and buys
  nothing.
- `macroexpand` when a macro's output confuses you — read what was EMITTED before theorising.
- Scratch `.wat` goes in `wat-scripts/scratch-pad/` and is loader-gated: green, or deleted.
- Do not commit.

## Your report

The diff shape; the three gate behaviours quoted from a real run; confirmation the handle sits in
`:ephemeral` and is born in `:init`; any STOP. No test-suite numbers — those are the orchestrator's.
