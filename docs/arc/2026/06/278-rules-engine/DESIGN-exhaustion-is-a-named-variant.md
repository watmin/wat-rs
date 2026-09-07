# DESIGN — exhaustion is a named variant

**Every retry ladder in the worker returns a CLOSED enum whose `Exhausted` variant the match must
name, and the two that still stop after three flat tries get the vis-bounded backoff the ack path
already has.** `wat-scripts/fanout/circuit.wat` only.

Re-draw of `exhaustion-cannot-be-discarded`, which was NOT STRUCK on STOP-1. The invariant is
unchanged; the mechanism is corrected.

## WHY — three hand-rollings of one shape, and two of them drop the flag

| ladder | line | on exhaustion | consequence |
|---|---|---|---|
| **check** | `:494-518` | returns `(Tuple 1 0)` → `gb-tick 1` → **`gave-back`** | **the terminal event.** Batch never acked; `vis` is 10¹² ns, so it is stranded past any bound |
| **mark** | `:560-576` | `second mm3` **discarded — no counter at all** | delivered-but-unrecorded. `Seen` cannot dedupe, so a later redelivery is a **duplicate** |
| **ack** | `:606-692` | vis-bounded backoff, `ack-retries` | fixed; proven live by `ack-retries=5` |

Measured twice on `2000 4 3 8192 true`:

```
mine:  [0/0][0/0][0/0][0/40]   ack-retries=5   gave-back=4
grok:  [0/0][0/0][0/0][0/10]   ack-retries=0   gave-back=1
```

★★★ **`gave-back` == stranded batches; unacked == gave-back × 10, in both runs.** `:518` is the
**only** producer of `gb-tick = 1`, so that counter has always measured exactly one thing — a
check-ladder abandonment. **Nothing is given back.** The name says the opposite of what happened.

## ⛔ WHAT THE PROBES ESTABLISHED — two refutations, precisely stated

Both measured this session, with the syntax verified against `wat/io.wat:40`:

| | |
|---|---|
| a **local** generic `fn` | instantiates its type parameter **once, at first use, then freezes**. `(twice 1 bump)` passes; `(twice "hi" shout)` then fails `expects i64; got String` |
| a **top-level** generic `defn` | instantiates **per call site** — both types work |
| a **parametric** enum, matched by keyword variant **in the EmptyEnv child** | illegal. The checker classifies it `MatchShape::Open` (`src/check.rs:6933` — *"no variant-constructor shape is determined"*) and accepts only hash-destructure or `_` |
| a **non-parametric** enum, same child | **legal, and live today** — `(:fanout::Seen::CheckResponse::Ok hits)` at `:519` runs every tick |

⚠ An earlier version of the first row was **wrong and is corrected on disk** — it read
`:wat::core::T` where the parameter is `T`, and blamed the language for a syntax error. See the
CORRECTION in `SCORE-exhaustion-cannot-be-discarded.md`.

★★★★ Together these say: the shared thing cannot be the loop (local `fn` freezes) and cannot be a
parametric type (`Open` in the child). **What survives is the invariant** — *exhaustion must be a
variant a match has to name* — and that needs a **closed** enum, one per peer type, because the peer
is in the payload.

## ⛔ THE ONE CONTRACT DECISION — two closed enums

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got       [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              reply <- :fanout::Seen::Reply]
  :Exhausted [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              attempts <- :wat::core::i64])

(:wat::core::defenum :fanout::QueueRetry :wat::enum::Pure  … Queue peer, Queue::Reply …)
```

Non-parametric, so both keyword-match inside the child. `SeenRetry` serves **check and mark**;
`QueueRetry` serves **ack**. A `Peer`-typed field in a plain enum is routine here —
`worker::State/q` and the queue's `:ephemeral [store <- Peer …]` both hold one.

⚠ **Two enums is not a concession.** What must not regrow is a **droppable flag**, and two closed
enums remove it at all three sites exactly as well as one parametric enum would have. What is lost
is DRY, not the invariant — and the invariant is the stone.

**And the two unfixed ladders get the proven loop:** retry with the inlined backoff (exponential,
jittered, capped) bounded by `vis-ns / 1000000` (`:606`). Same derivation, unchanged: after vis
expiry the message is visible again and retrying is pointless — which is also why chaos runs
(`vis = 200 ms`) keep exercising redelivery **by construction**.

### `gave-back` is renamed

It measures check-ladder exhaustion and asserts the opposite. Confined to `circuit.wat`, **asserted
in no test** — the rename is one file and no churn.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

★ **The gate is binary and currently failing in both our hands:** `2000 4 3 8192 true` completes, or
it does not. The mechanism is **measured**, not inferred — `gave-back` names the exact event and the
exact batch count, twice.

⚠ **Two rungs, deliberately.** The enum is the rung where the mistake cannot be *expressed*; the
backoff fixes the instances that exist. Enum alone changes no behaviour; backoff alone leaves the
shape free to regrow a fourth time.

⚠ **This does NOT fix the class.** The duplication survives: the backoff formula stays in three
copies (`:1204` helper, `:1540` publisher child, `:675` worker child) with `100` hardcoded in two,
because an `EmptyEnv` child cannot see the file's own `defn`s. Saying otherwise would be the
dishonest version of this stone.

⚠ **It does not explain the 25 % slope** (4348 → 3244 across n=100→1000). That was measured with
`gave-back=0` on every point, so it has a different cause and stays open.

## OUT OF SCOPE — REJECTED

- **One shared combinator, and promotion to `wat/`.** A generic **top-level** `defn` would serve
  both peer types, and the stdlib **is** visible to the child (`circuit.wat` makes 71 `:wat::` calls
  from inside service impls) — so this is the real fix for the duplication. It is **rejected here on
  the precedent's own terms**: `wat-scripts/queue/README.md` says *"promoted to `wat/queue.wat` when
  it demonstrates excellence — **the builder's ruling, never a side effect**."* The code that would
  move is currently broken. **This stone is how that promotion gets earned, not a substitute for it.**
- **One parametric enum.** Refuted by measurement; `Open` in the child.
- **Hash-destructure arms.** The checker suggests them, and they defeat the stone: `MatchShape::Open`
  determines no variant shape and tracks exhaustiveness by wildcard alone, so an Open match is
  **exactly as droppable as the `bool` it replaces**, and worse for looking exhaustive.
- **Shortening `vis` on normal runs.** Would net a stranded batch, but changes what the circuit
  tests. A different ruling.
- **The receive path's 1000 ms deadline** (`:452`). A receive timeout leaves the message visible,
  which is already safe.
