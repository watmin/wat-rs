# BRIEF — ②a v2, RING 1: propagate the memory types from the four walkers

Design: `DESIGN-STONE-2a-v2-propagate-from-the-walkers.md`. Merge `fe7700f0e` split
`walk-sorted-ids` into four monomorphic walkers, each carrying a concrete memory type. **Ring 1 is
the six functions those walkers call.** Every annotation you write is already decided by a caller —
this brief names each one and its source.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** No `cargo`, in any
form. `./target/release/wat --check <file>` is prebuilt and CURRENT for the *checker*, but ⚠ `wat/**`
is baked into the binary via `include_str!` (`src/stdlib.rs`), so it CANNOT validate your on-disk
`wat/` edits — a prior stone escaped through exactly that gap. The orchestrator's build is the only
verification. Foreground everything; ending your turn ends you. Do not commit, push, stash, or revert.

## ★ The method: every type is COPIED from a call site, never inferred, never read from prose

`wat/rete/oracle/fire.wat` already declares these (merge `723936550`, floor green 4855/4855):

```wat
walk-alpha-ids   acc  <- PersistentMap<i64, PersistentVector<wat::rete::Element>>   ; and its ->
walk-beta-ids    amem <- PersistentMap<i64, PersistentVector<wat::rete::Element>>
                 acc  <- PersistentMap<i64, PersistentVector<wat::rete::Token>>     ; and its ->
walk-filter-ids  amem <- …Element>>   acc <- …Token>>                               ; and its ->
walk-prod-ids    bmem <- …Token>>     acc <- PersistentMap<i64, PersistentVector<wat::core::Record>>
```

⚠ **Write the types in TODAY'S angle form**, exactly as the walkers do. The bracket migration is a
separate, later stone; do not mix them.

## The work — 16 annotations, each with its source

| # | file:line | fn | param | ← copied from |
|---|---|---|---|---|
| 1 | `pass.wat:40` | `activate-alpha` | `alpha-mem` | `walk-alpha-ids.acc` |
| 2 | ″ | ″ | `->` | `walk-alpha-ids.->` |
| 3 | `pass.wat:133` | `root-join-pass` | `alpha-mem` | `walk-beta-ids.amem` |
| 4 | ″ | ″ | `beta-mem` | `walk-beta-ids.acc` |
| 5 | ″ | ″ | `->` | `walk-beta-ids.->` |
| 6 | `pass.wat:326` | `hash-join-pass` | `alpha-mem` | `walk-filter-ids.amem` |
| 7 | ″ | ″ | `beta-mem` | `walk-filter-ids.acc` |
| 8 | ″ | ″ | `->` | `walk-filter-ids.->` |
| 9 | `pass.wat:679` | `filter-pass` | `alpha-mem` | `walk-filter-ids.amem` |
| 10 | ″ | ″ | `beta-mem` | `walk-filter-ids.acc` |
| 11 | ″ | ″ | `->` | `walk-filter-ids.->` |
| 12 | `pass.wat:787` | `production-pass` | `beta-mem` | `walk-prod-ids.bmem` |
| 13 | ″ | ″ | `prod-mem` | `walk-prod-ids.acc` |
| 14 | ″ | ″ | `->` | `walk-prod-ids.->` |
| 15 | `accum-pass.wat:28` | `accumulate-pass` | `bm` | `walk-filter-ids.acc` — ⚠ see below |
| 16 | ″ | ″ | `->` | ″ |

⚠ Confirm every line number by matching the surrounding code, not by trusting it.

## ⛔ NOT in ring 1 — leave these bare

- **`network`** and **`facts`**. They are bare in the WALKERS too, so nothing upstream decides them.
  They wait for a ring that reaches them. Touching them is guessing.
- **`bindings`, `ext`, `m`, `params`, `km`, `nb`.** Separately unresolved. `:wat::core::Value` was
  tried and is **wrong** — rete compares binding values with `<`/`>`, keys on them, and `conj`s them
  into vectors, all of which an opaque `Value` refuses (arc 278 R7, `src/types.rs:1160`). Leave bare.
- **Ring 2 and outward** (`activate-fact`, `append-token`, …). A later stone; ring 1 must floor first.

## ⚠ `bm` at `accum-pass.wat:28` — read the body, the name is not evidence

In the pre-merge file, two `bm` sites were local **bindings** accumulators while 25 siblings were beta
memory. A rider caught it by reading bodies; a name-based census would have mistyped both.

Row 15 says `bm` here is the beta memory threaded from `walk-filter-ids`. **Verify that at the call
site before writing it.** If the body shows it is a bindings accumulator, that is STOP-2 — report it
and leave it bare.

## The contract decision

> **Every annotation is COPIED from a named call site. If you cannot name the caller whose declared
> type you copied, do not write the annotation.**

No inference, no comment, no sibling that looks similar. ⚠ The `alpha-memory`/`beta-memory` doc
comments are **known stale** — they describe a nested `{join-bindings → …}` shape that does not exist.
Prose is not a source in this stone.

## Blast radius

`wat/rete/oracle/pass.wat` · `wat/rete/oracle/accum-pass.wat`. Nothing else. No `src/`, no `tests/`,
no other `.wat`.

## STOP triggers — each rejects; none is a fallback

1. A parameter's type cannot be traced to a caller's declared type. STOP for that row; leave it bare.
2. `bm` at `accum-pass.wat:28` is a bindings accumulator, not beta memory. STOP; report; leave bare.
3. A call site passes a memory whose declared type CONTRADICTS another call site of the same fn.
   STOP — that is a real finding and means the fn is polymorphic, exactly as `walk-sorted-ids` was.
4. You need to touch `network`, `facts`, `bindings`, `src/`, or `tests/`. STOP.

## Acceptance criteria

- 16 annotations written, each in today's angle form, each matching its source verbatim.
- Every one reported with the caller it was copied from.
- `network`, `facts` and the bindings family still bare.
- Only `pass.wat` and `accum-pass.wat` touched.
- The orchestrator's floor is the verification; do not claim green.
