# NOTE — the cache LRU panics on a value that arrives from durable storage

**Filed:** 2026-08-28, by the grok-rete agent, at the builder's direction
(*"the cache lru… make a note on this… drop it into arc 109… this is unrelated to rete/278"*).
**Home:** arc 109, because `src/rust_deps/` is its territory and the cache is not rete.
**Status:** RULED ON THE MERITS (below), awaiting **MANDATE**. Nothing here is started.
**Ground:** `grok-rete` @ `f1e112562`. Every citation re-checked against the tree on the day.

---

## What panics

`src/rust_deps/cache.rs` carries three `panic!` guards behind two distinct questions:

| site | guard | class |
|---|---|---|
| `Lru::new` (`:101`) | `capacity <= 0` | **fallible input** — see below |
| `Lru::put` (`:124`) | key is not hashable | caller bug |
| `Lru::get` (`:136`) | key is not hashable | caller bug |

The module doc (`:46`) names them as a deliberate pair. Stone 1 surfaced them as a question and
left them to *"a later stone"* — prose with no owner and no re-read, which is how the premise below
rotted for a month without anyone learning.

## The deferral's stated reason is no longer true

Stone 1's ground was that `#[wat_dispatch]` could not marshal a method-internal error back to wat.
At this HEAD it can, and has been able to for some time. `src/rust_deps/sqlite.rs:11-17` states the
mechanism outright: `Result<T, E>` and tuples already carry blanket `ToWat`/`FromWat` impls
(`src/rust_deps/marshal.rs`), and `#[wat_dispatch]`'s codegen handles them with **zero macro
changes** — including `Result<Self, E>` for a constructor.

So the conversion is mechanically available today. What is left is a design call, which is why this
is a ruling and not a cleanup.

## ★ THE MERITS — and the two panics do NOT answer the same way

The question was posed as *"does the no-hidden-failures law reach a **programming-error** input, or
stop at a **fallible** one?"* Posed over all three sites at once, both readings look defensible.
Split per site, one of them stops being a judgment call:

**`Lru::new`'s capacity is NOT a caller-supplied constant. It arrives from durable storage.**
`wat/cache.wat:129-134` is the contract:

```
;; ─── the one contract decision — durable holds the SPEC, ephemeral holds the HANDLE ───
;;   `:durable   [capacity <- i64]`   — plain EDN, the SPEC the resource is rebuilt from.
;;   `:ephemeral [cache <- (Lru :- [K V])]`  — the live handle, born inside `:init` by calling
;;                                             `Lru::new` on the durable capacity.
```

The capacity **crosses a serialization boundary and comes back as data**. A `0` in a stored durable
record — hand-edited EDN, a truncated write, a schema change, a migration — reaches `Lru::new` at
**rehydration** and panics the process. That is exactly the shape sqlite's verbs have (a disk can be
missing), and it is not a caller bug: no caller is in the frame when it fires.

**`put`/`get`'s non-hashable key is a different animal.** The key is supplied at the call site, and
the checker already rejects an opaque-typed key at most of them. Nothing rehydrates it. That one is
a genuine programming error and the panic is defensible.

**RECOMMENDATION: convert `Lru::new` to `Result<Self, E>`; LEAVE `put`/`get`.**
That is the reading the substrate itself argues for, and it shrinks the blast radius substantially
versus converting all three.

## Cost, honestly

Converting `Lru::new` changes a **shipped public surface**: every caller matches a `Result`, and
`wat/cache.wat`'s `lru-svc` moves in the same breath — including the durable-record rebuild path,
which is the very path that motivates the change. **Not a one-file strike.**

Leaving `put`/`get` alone means the module doc's "two guards panic" paragraph must be rewritten
rather than deleted: it currently presents both as one decision, and after this they are two.

## What NOT to do

- **Do not convert all three for symmetry.** The record would then say the law reaches programming
  errors, which is a much larger claim than the evidence here supports, and it triples the surface
  churn for the two sites that do not need it.
- **Do not "fix" it by clamping** — `capacity.max(1)` or similar. A stored `0` means the durable
  record is wrong; silently inventing a capacity hides that from whoever has to find it later.
- **Do not leave it as prose again.** Its first deferral was a source comment saying "a later
  stone", with no owner and no re-read, and that is precisely why its premise could go stale
  unnoticed. This note is the re-readable row; if it is ruled LEAVE, write that here.

## Citations

| what | where |
|---|---|
| the three panics | `src/rust_deps/cache.rs:101`, `:124`, `:136` |
| the module doc presenting them as one pair | `src/rust_deps/cache.rs:46` |
| durable holds the SPEC, ephemeral the HANDLE | `wat/cache.wat:129-134` |
| `Result<T, E>` marshals with zero macro changes | `src/rust_deps/sqlite.rs:11-17` |
| blanket `ToWat`/`FromWat` | `src/rust_deps/marshal.rs` |
| the original bounded deferral row (①) | `docs/arc/2026/06/278-rules-engine/NEXT-STRIKES-theater-hunt.md` § "TRACKED DECISIONS" |
