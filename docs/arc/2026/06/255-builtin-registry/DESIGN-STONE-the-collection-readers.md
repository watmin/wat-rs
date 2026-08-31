# DESIGN — STONE: the collection readers get homes

> **Builder, 2026-08-31:** *"collection readers next"*
>
> ⚠ I offered "the collection readers (6)". **Two of the six are W7 and are not in this stone** —
> found in pre-flight, not by a rider hitting them.

## The six, measured — and the split

| verb | runs caller code? | verdict |
|---|---|---|
| `assoc` | no | ✅ in scope |
| `conj` | no — routes through `StreamContainer::has_append()`, the capability gate that **refuses** a lazy receiver rather than forcing it | ✅ in scope |
| `drop` | no | ✅ in scope |
| `take` | no | ✅ in scope |
| **`find-last-index`** | ⛔ **YES** — `apply_function(func.clone(), vec![x.clone()], …)` on a caller predicate | ⛔ W7 |
| **`seqable->stream`** | ⛔ **YES** — forces a thunk; named in the W7 NOTE by name | ⛔ W7 |

★ **`find-last-index` is a HOF wearing a reader's name.** The 44-unhomed worklist once filed it as
"INTRINSIC-READY"; it applies a caller's predicate, which is the exact mechanism that put
`map`/`mapv`/`filter`/`foldl` behind a language ruling.

## ⚠ One subtlety that must not be misread

`drop` and `take` **return** `Value::wat__stream__Stream(lazy_take_stream(…))` — they *construct* a
lazy stream. **Constructing a thunk is not running one.** Nothing is forced, no caller code executes,
and the deferred work belongs to whoever forces it later. They are readers, not forcers — which is
exactly what separates them from `seqable->stream` two rows above.

## THE ONE CONTRACT DECISION — pinned

**"Runs code it did not write" is the scope test, and it is measured at the body — never inferred
from the verb's name or its worklist category.** Two of these six read like collection readers and
are not.

## The rulings — from the bodies, by the rider

All four are expected `Pure ∧ Deterministic`; each has exactly **one** `return Err` (its arity
guard), which retires on homing. ⚠ **`@Totality` is the rider's to measure per verb**, not to copy
across the four: a `TypeMismatch` on a wrong receiver type is a different question from a domain
hole, and `assoc`/`conj` reach container-capability gates that `drop`/`take` do not.

## ★ A MIXED PREDICTION, falsifiable in both directions

```
assoc   env.register scheme: YES  ->  NO debt row expected
conj    env.register scheme: YES  ->  NO debt row expected
drop    env.register scheme: NO   ->  a debt row expected
take    env.register scheme: NO   ->  a debt row expected
```

**FROZEN_CHECKER_DEBT_LEDGER 62 → 64.** Better than a uniform prediction: if all four need rows, or
none do, the measurement is wrong in a way a uniform guess would have hidden.

## Out of scope = REJECTED (not deferred)

- **`find-last-index` and `seqable->stream`** — W7. ★ **And `find-last-index` is now potentially
  unblockable by the `sort$native` treatment**: impose Pure ∧ Deterministic on the predicate at its
  door, which the classifier can now check through a closure. That is a real follow-on and a
  separate stone — it changes behaviour, where this one does not.
- **The rest of `KNOWN_UNREVIEWED`** (45 today; 41 after this).

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the four that run no caller code** | YES | YES | YES | YES | ✅ **ADMITTED** |
| all six, ruling the two W7 verbs Pure | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| all six, imposing on `find-last-index` here | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| four, but copy one `@Totality` across them | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **six-with-W7-Pure Honest? NO** — measured, both run caller code; `Pure` would be falsifiable in
  one line, exactly as `sort$native`'s was before its gate.
- **impose-here Simple? NO** — four mechanical homings plus a behavioural gate in one stone; a red
  could not be attributed.
- **copy-the-totality Honest? NO** — the four reach different failure surfaces. A declaration copied
  from a neighbour is the defect this arc keeps finding.

## Acceptance

| what | command | expected |
|---|---|---|
| the four are registered | `lookup_entry` each | `Some` |
| each `@Totality` is its own | the four declarations | measured per verb, cited |
| the ratchet | `KNOWN_UNREVIEWED` | 45 → **41** |
| the mixed prediction | `FROZEN_CHECKER_DEBT_LEDGER` | 62 → **64**, and only `drop`/`take` |
| behaviour unchanged | `assoc` · `conj` · `drop` · `take` on a vector and a stream | as today |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
