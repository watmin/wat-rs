# Arc 119 — Sonnet Brief: HolonLRU HologramCacheService reshape (steps 4+5)

**Status:** durable record of the brief sent to sonnet for the
HolonLRU substrate reshape. Companion to BRIEF-LRU-RESHAPE.md;
most disciplines carry over unchanged, only the differences are
spelled out below.

## Provenance

Steps 4+5 of arc 119's execution checklist. Lands AFTER steps 2+3
(LRU CacheService reshape) which shipped via
BRIEF-LRU-RESHAPE.md. The HolonLRU surface mirrors the LRU
surface exactly with one substitution — `K = V = HolonAST` — and
one wrinkle: HolonLRU is still grouped under
`HologramCacheService::*` (K.holon-lru flattens later in arc 109).

## Goal

One file edit:
`/home/watmin/work/holon/wat-rs/crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.

After this slice ships green, both Pattern B services share the
locked symmetric batch protocol; arc 119 step 6 (workspace test
baseline) and step 7 (consumer sweep) follow.

## Anchor docs (read in order)

1. **`BRIEF-LRU-RESHAPE.md` (in this same directory)** — read in
   full. The disciplines, validation gate, and constraints are
   identical. Don't re-derive; copy.
2. `crates/wat-lru/wat/lru/CacheService.wat` (post-LRU-reshape,
   committed at HEAD) — the **canonical reference shape**. Read
   lines 50-110 (typealiases + Request enum) and lines 200-280
   (handle + verbs). Mirror this shape with K=V=HolonAST.
3. `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`
   §§ "Substrate work scope" → `:wat::holon::lru::*`,
   "Execution checklist" steps 4+5.

## Differences from the LRU brief

### 1. Concrete, NOT parametric

LRU is `<K,V>` polymorphic. HolonLRU is concrete:
**K = V = `:wat::holon::HolonAST`** throughout. No type parameters
on any decl. Wherever the LRU file says `<K,V>` or uses `K`/`V`,
HolonLRU writes `wat::holon::HolonAST` directly.

Implication: the parametric-user-enum-match path arc 120 fixed
**does not** affect HolonLRU directly (HolonLRU's `Request` enum
is non-parametric). But the path is exercised by LRU at HEAD,
so we know it works.

### 2. Symbols stay under `HologramCacheService::` prefix

K.holon-lru (a future arc 109 slice) will flatten the
`HologramCacheService::*` prefix to bare `:wat::holon::lru::*`.
Your job is **NOT** to flatten — leave the prefix intact. Mint
new typealiases under `:wat::holon::lru::HologramCacheService::*`
alongside the existing ones.

### 3. Backing primitive is `HologramCache`, not `LocalCache`

In the LRU file, the cache is `:wat::lru::LocalCache<K,V>` with
methods `LocalCache::get` and `LocalCache::put`. In the HolonLRU
file, the cache is `:wat::holon::lru::HologramCache` with methods
`HologramCache/get` and `HologramCache/put` (and possibly
others — check `crates/wat-holon-lru/wat/holon/lru/HologramCache.wat`
for the actual signatures before using them).

Important: `HologramCache/put` may have a DIFFERENT return type
than `LocalCache::put` (which returns `Option<(K,V)>` for
eviction). Verify before mapping over entries; adapt the
discarded-eviction binding to whatever HologramCache/put actually
returns.

### 4. Existing file's verb namespace

Check the existing file's verb-naming convention before writing
new verbs. The LRU file uses bare `:wat::lru::get` /
`:wat::lru::put` because K.lru already flattened. HolonLRU's
existing file may use `:wat::holon::lru::HologramCacheService/get`
or similar with the grouping noun in the path. **Mirror what's
already there.** If the existing file has no client-side verbs at
all, mint them under the path convention the rest of the file's
existing verbs use.

## Required disciplines

Identical to BRIEF-LRU-RESHAPE.md §§ "Required disciplines":

1. **Send pattern** — one `Result/expect -> :unit`, no Option
   layer (send returns `Result<unit, ThreadDiedError>`).
2. **Recv pattern** — `Option/expect` wrapping `Result/expect ->
   :Option<T>` (recv returns `Result<Option<T>, ThreadDiedError>`).
3. **Inner-colon antipattern** — bare tuples inside generics:
   `Vector<Option<(K,V)>>`, never `Vector<Option<:(K,V)>>`. Same
   rule applies to `Vector<Option<HolonAST>>` etc. — colon prefix
   only at the outermost type position.

Read those sections of BRIEF-LRU-RESHAPE.md for the worked code
patterns; copy directly.

## Target shape

Mirror the post-LRU-reshape file. Concrete substitutions:

### Typealiases (under `HologramCacheService::*` prefix)

- **Mint** `Entry`:
  ```scheme
  :wat::holon::lru::HologramCacheService::Entry
    = :(wat::holon::HolonAST, wat::holon::HolonAST)
  ```
- **Mint** `PutAckTx/Rx/Channel`:
  ```scheme
  :wat::holon::lru::HologramCacheService::PutAckTx
    = :wat::kernel::Sender<wat::core::unit>
  ;; (and Rx, Channel correspondingly)
  ```
- **Reshape** `GetReplyTx`/`GetReplyRx`/`GetReplyPair` bodies —
  was `Sender<Option<HolonAST>>`, becomes
  `Sender<Vector<Option<HolonAST>>>`. Same names; widened bodies.
  Note: `GetReplyPair` keeps its name (K.holon-lru renames to
  `GetReplyChannel` later).
- **Reshape** Request enum:
  ```scheme
  (:wat::core::enum :wat::holon::lru::HologramCacheService::Request
    (Get  (probes  :wat::core::Vector<wat::holon::HolonAST>)
          (reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx))
    (Put  (entries :wat::core::Vector<wat::holon::lru::HologramCacheService::Entry>)
          (ack-tx   :wat::holon::lru::HologramCacheService::PutAckTx)))
  ```
- `ReqTx`, `ReqRx`, `ReqTxPool` keep names; auto-pick up new
  Request enum.

### Driver `handle`

Match the new enum.

- **Get arm**: `map` probes via `HologramCache/get` building
  `Vector<Option<HolonAST>>`; fold for hits; send batch on
  reply-tx using send pattern; bump stats.
- **Put arm**: `map` entries calling `HologramCache/put cache k v`
  for each; bind discarded result to `_` with whatever type
  `HologramCache/put` returns (verify first); send `()` on
  ack-tx using send pattern; bump stats.

### Verbs

If the existing file has client-side helpers (`get` / `put`
analogues), reshape them to:

```scheme
;; (signature mirrors LRU's get/put but with concrete HolonAST
;;  and the existing file's verb-naming convention)
```

Probes-vec → `Vector<Option<HolonAST>>` reply via recv pattern;
entries-vec → unit ack via recv pattern (T = unit).

If the existing file has NO client-side helpers, mint them under
the file's existing verb-naming convention (mirror what other
verbs in the file do). Verify by reading the file before
writing.

## What stays unchanged

Same as LRU: Stats, Report, MetricsCadence, Reporter, State,
Step, null-helpers, spawn, loop, loop-step, tick-window — all
keep their existing structure.

## Header doc comment

Update to describe arc 119 batch protocol (same prose as LRU's
update, but reference HolonLRU specifics). Cite arc 119 + the
batch convention in CONVENTIONS.md.

## Validation gate

Same as LRU brief, swap crate name:

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release -p wat-holon-lru                 # Rust compiles
cargo test  --release -p wat-holon-lru --lib           # substrate parses + type-checks
```

Both must succeed. `cargo test -p wat-holon-lru --lib` with
`0 tests run` is green for this gate (the lib parses substrate
wat at SymbolTable init; that's what we're checking).

**Do NOT** run `cargo test -p wat-holon-lru` without `--lib` —
wat-tests will fail; that's step 7's territory.

## Constraints

- ONE file edit only:
  `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
- Do NOT touch `crates/wat-lru/` (already shipped)
- Do NOT touch `crates/wat-holon-lru/wat-tests/` (step 7)
- Do NOT touch any other crate
- Do NOT add dependencies, edit Cargo.toml, or touch `src/*.rs`

## Reporting back

Same format as LRU brief:
1. `git status --short` — file list (should show ONE modified
   file)
2. `git diff --stat` — line counts
3. `cargo build --release -p wat-holon-lru` outcome
4. `cargo test --release -p wat-holon-lru --lib` outcome — show
   the `test result:` line(s)
5. Any judgment calls beyond the brief (especially: what
   `HologramCache/put` returns; what verb names the existing
   file uses)

## Cross-references

- `BRIEF-LRU-RESHAPE.md` (this directory) — full disciplines,
  validation gate, constraints. Read this first.
- `crates/wat-lru/wat/lru/CacheService.wat` (post-LRU-reshape,
  HEAD) — the canonical shape to mirror with K=V=HolonAST.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the arc
  this brief executes against.
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  — the file to edit.
- `crates/wat-holon-lru/wat/holon/lru/HologramCache.wat` —
  reference for HologramCache/get / put signatures.
