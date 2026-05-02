# Arc 119 — Sonnet Brief: LRU CacheService reshape (steps 2+3)

**Status:** durable record of the brief sent to sonnet for the
LRU substrate reshape. Same brief stays in this file as the
reference for re-attempts, post-mortems, and the future
HOLON-LRU companion brief.

## Provenance

First attempt at arc 119 substrate work used a thinner brief and
hit four type-check errors — three sonnet bugs (missing Option
layer on recv chains; illegal `:(K,V)` inner-colon antipattern)
plus one substrate gap (parametric user-defined enum match
patterns failing to type-check, fixed in arc 120 commit
`4618d36`). After the orchestrator's "trust but verify" caught
the breakage via `cargo test --release --workspace`, the wat
substrate edits were reverted to canonical pre-119 state
(working tree clean; baseline 1476/0/2 → 1479/0/0 after arc 120).

This brief encodes everything the first attempt missed: precise
substrate signatures, three required disciplines spelled out
with code patterns, and a stronger validation gate
(`cargo test --lib`, not just `cargo build`).

## Goal

One file edit: `/home/watmin/work/holon/wat-rs/crates/wat-lru/wat/lru/CacheService.wat`.

Steps 2+3 of arc 119's execution checklist (see DESIGN.md).
After this slice ships green, K=V=HolonAST mirror lands in
HologramCacheService.wat (steps 4+5, separate brief).

## Anchor docs (read in order)

1. `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`
   §§ "The fix — symmetric batch protocol", "Substrate work
   scope" → `:wat::lru::*`, "Execution checklist" steps 2+3.
2. `docs/CONVENTIONS.md` § "Batch convention" — the
   substrate-wide rule arc 119 enforces.
3. `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" — the
   discipline being protected.

## Substrate context (compile-correctness)

- **Arc 120 just shipped (commit `4618d36`).** Parametric
  user-defined enums now type-check correctly when matched on.
  `Request<K,V>` enum + `(:wat::lru::Request::Get probes
  reply-tx)` pattern-arms unify against scrutinee type
  `:wat::lru::Request<K,V>`. This was previously broken; arc 119
  cannot resume without arc 120.
- **Arc 115 — inner-colon antipattern.** `:(K,V)` and `:T` are
  illegal inside `<>`, `()`, or `fn(...)`. Inner type args are
  bare. Example: `Vector<Option<(K,V)>>` is correct;
  `Vector<Option<:(K,V)>>` is parser-rejected. The colon prefix
  lives at the outermost type position only.
- **Arc 111 + 113 — kernel comm signatures:**
  - `:wat::kernel::send` returns `:Result<unit, ThreadDiedError>`
    (NO Option layer)
  - `:wat::kernel::recv` returns `:Result<Option<T>, ThreadDiedError>`
    (HAS Option layer — Some=msg, None=clean disconnect)

## Required disciplines

These three are the gotchas the first attempt missed. Each has
a worked code pattern; copy the shape directly.

### 1. Send pattern — one Result/expect, no Option layer

`send` returns `Result<unit, ThreadDiedError>`. Unwrap with one
Result/expect; the inner is unit, no Option to peel.

```scheme
((_send :wat::core::unit)
 (:wat::core::Result/expect -> :wat::core::unit
   (:wat::kernel::send tx val)
   "X: tx disconnected — peer died?"))
```

### 2. Recv pattern — Result/expect WRAPPING Option/expect (two nested levels)

`recv` returns `Result<Option<T>, ThreadDiedError>`. Result/expect
unwraps the Result (panic on peer thread death); Option/expect
unwraps the Option (panic on clean channel close).

```scheme
(:wat::core::Option/expect -> :T
  (:wat::core::Result/expect -> :wat::core::Option<T>
    (:wat::kernel::recv rx)
    "X: rx peer died — protocol violation")
  "X: rx channel closed — peer dropped tx?")
```

For batch get's reply-rx, `T = wat::core::Vector<wat::core::Option<V>>`.

### 3. Tuple inside generic args is bare

```scheme
:wat::core::Vector<wat::core::Option<(K,V)>>          ;; CORRECT
:wat::core::Vector<wat::core::Option<:(K,V)>>         ;; ILLEGAL — arc 115
```

The colon-prefix `:(K,V)` form is legal at top-level type
positions (e.g., `:wat::core::typealias :wat::lru::Entry<K,V> :(K,V)`)
but illegal nested inside another generic's args.

## Target shape (locked by gaze, three passes; from DESIGN.md)

### Typealiases

- **Retire** `Body<K,V>` entirely.
- **Mint** `Entry<K,V> = :(K,V)` (top-level — `:` is at outermost
  position).
- **Mint** `PutAckTx`, `PutAckRx`, `PutAckChannel`:
  ```scheme
  :wat::lru::PutAckTx       = :wat::kernel::Sender<wat::core::unit>
  :wat::lru::PutAckRx       = :wat::kernel::Receiver<wat::core::unit>
  :wat::lru::PutAckChannel  = :(wat::lru::PutAckTx,wat::lru::PutAckRx)
  ```
- **Reshape** `ReplyTx<V>` body — was `Sender<Option<V>>`, becomes:
  ```scheme
  :wat::lru::ReplyTx<V>     = :wat::kernel::Sender<wat::core::Vector<wat::core::Option<V>>>
  ```
  Same name; widened body for batch return.
- **Reshape** `ReplyRx<V>`, `ReplyChannel<V>` widen
  correspondingly.
- **Reshape** `Request<K,V>` from typealias-tuple to a
  `:wat::core::enum` declaration:
  ```scheme
  (:wat::core::enum :wat::lru::Request<K,V>
    (Get  (probes  :wat::core::Vector<K>)
          (reply-tx :wat::lru::ReplyTx<V>))
    (Put  (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
          (ack-tx   :wat::lru::PutAckTx)))
  ```
- `ReqTx<K,V>`, `ReqRx<K,V>`, `ReqChannel<K,V>` keep names; auto-pick
  up the new enum-`Request<K,V>` body. No edits needed.

### Driver `:wat::lru::handle<K,V>`

Match the new enum:

- **Get arm**: `(:wat::lru::Request::Get probes reply-tx)` —
  `map` probes via `:wat::lru::LocalCache::get` building
  `Vector<Option<V>>`; fold for hit count; **send the batch
  result on `reply-tx` using send pattern (one Result/expect)**;
  bump stats: `lookups += len(probes)`, hits/misses computed
  from the result vec.
- **Put arm**: `(:wat::lru::Request::Put entries ack-tx)` —
  `map` entries; for each `(k,v)` Entry, call `LocalCache::put
  cache k v` (returns `Option<(K,V)>` eviction; discard).
  **Bind the discarded result to `_` with type
  `:wat::core::Vector<wat::core::Option<(K,V)>>` — the inner
  tuple is BARE, no `:` prefix.** Send `()` on `ack-tx` using
  send pattern; bump stats: `puts += len(entries)`.

### Verbs

```scheme
(:wat::core::define
  (:wat::lru::get<K,V>
    (req-tx :wat::lru::ReqTx<K,V>)
    (reply-tx :wat::lru::ReplyTx<V>)
    (reply-rx :wat::lru::ReplyRx<V>)
    (probes :wat::core::Vector<K>)
    -> :wat::core::Vector<wat::core::Option<V>>)
  ;; build (Request::Get probes reply-tx)
  ;; send on req-tx using send pattern
  ;; recv on reply-rx using recv pattern (Result wraps Option)
  ;; T for the recv chain is :Vector<Option<V>>
  ...)

(:wat::core::define
  (:wat::lru::put<K,V>
    (req-tx :wat::lru::ReqTx<K,V>)
    (ack-tx :wat::lru::PutAckTx)
    (ack-rx :wat::lru::PutAckRx)
    (entries :wat::core::Vector<wat::lru::Entry<K,V>>)
    -> :wat::core::unit)
  ;; build (Request::Put entries ack-tx)
  ;; send on req-tx using send pattern
  ;; recv on ack-rx using recv pattern (T = unit)
  ;; return ()
  ...)
```

Note: `put`'s signature changed shape. It no longer takes
`reply-tx`/`reply-rx` — it takes `ack-tx`/`ack-rx` (PutAck
family). Variant-scoped channel naming per gaze: GET uses
Reply* (data back), PUT uses Ack* (unit-ack release).

## What stays unchanged

- `Stats`, `Report`, `MetricsCadence`, `Reporter`, `State`,
  `Step` typealiases / structs.
- `null-reporter`, `null-metrics-cadence`, `Stats/zero`.
- `tick-window`, `loop`, `loop-step`, `spawn`. The driver-loop
  wiring stays structural; only `handle`'s body changes per the
  new enum.
- The Reporter+MetricsCadence contract per arc 078 stays.

## Header doc comment

Update to describe arc 119 batch protocol:
- Request is now an enum (Get | Put), not a tagged-tuple
- Get carries Vec<K> probes, returns Vec<Option<V>>
- Put carries Vec<Entry<K,V>>, returns unit-ack
- Variant-scoped channel families: Reply* for Get (data-back,
  Pattern B), PutAck* for Put (unit release, Pattern A)
- Reference arc 119 + the batch convention in CONVENTIONS.md

## Validation gate (CRITICAL)

`cargo build` is NOT enough. Wat parses + type-checks at
runtime; build only confirms Rust syntax. Both gates must pass:

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release -p wat-lru                 # Rust compiles
cargo test  --release -p wat-lru --lib           # substrate parses + type-checks
```

Both must succeed. The `--lib` flag runs ONLY the lib's unit
tests (not the wat-tests, which use the OLD API and will fail —
that's step 7's territory). If `cargo test -p wat-lru --lib`
fails with a wat type error, sonnet's wat is broken and needs
fixing in-place; do NOT proceed without surfacing every error.

If `cargo test -p wat-lru --lib` passes silently with `0 tests
run`, that's still green for this gate (the lib has zero unit
tests; the substrate wat parsing is what we're checking by
exercising the loader). What matters is no Rust panic, no
substrate parse error during `stdlib_loaded`.

**Do NOT** run `cargo test --release -p wat-lru` without
`--lib` — that runs wat-tests which WILL fail and is expected.

## Constraints

- ONE file edit only:
  `crates/wat-lru/wat/lru/CacheService.wat`
- Do NOT touch `crates/wat-holon-lru/` (steps 4+5)
- Do NOT touch `crates/wat-lru/wat-tests/` (step 7)
- Do NOT touch any other crate
- Do NOT add dependencies, edit Cargo.toml, or touch
  `src/check.rs` / `src/types.rs`

## Reporting back

When done (or blocked):
1. `git status --short` — file list (should show ONE modified
   file)
2. `git diff --stat` — line counts
3. `cargo build --release -p wat-lru` outcome
4. `cargo test --release -p wat-lru --lib` outcome — show the
   `test result:` line(s)
5. Any judgment calls beyond this brief

The orchestrator runs `git diff --stat` independently to
verify against your file list per the trust-but-verify
discipline.

## Working directory

Repo root: `/home/watmin/work/holon/wat-rs/`. All cargo
commands work from there directly; `--manifest-path` not
needed if cwd is the repo root.

## Cross-references

- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the
  arc this brief executes against.
- `docs/arc/2026/05/120-parametric-user-enum-match/DESIGN.md`
  — the substrate gap fix that unblocks this brief.
- `crates/wat-lru/wat/lru/CacheService.wat` — the file to edit.
