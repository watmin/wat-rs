# DESIGN — exhaustion cannot be discarded

**Every retry ladder in the worker returns an outcome the caller must match, and the two that still
give up after three flat tries get the vis-bounded backoff the ack path already has.**
`wat-scripts/fanout/circuit.wat` only.

## WHY — three hand-rollings of one shape, and two of them drop the flag

The worker's tick arm holds **three** retry ladders within eighty lines:

| ladder | line | on exhaustion | consequence |
|---|---|---|---|
| **check** | `:513-518` | returns `gb-tick 1` → **`gave-back`** | **the terminal event.** Batch never acked; `vis` is 10¹² ns, so it is stranded past any bound |
| **mark** | `:574-576` | `second mm3` **discarded — no counter at all** | delivered-but-unrecorded. `Seen` cannot dedupe, so a later redelivery is a **duplicate** |
| **ack** | `:591+` | vis-bounded backoff, `ack-retries` | fixed by the previous stone |

Measured, twice, on the n=2000 fill-first run:

```
mine:  [0/0][0/0][0/0][0/40]   ack-retries=5   gave-back=4
grok:  [0/0][0/0][0/0][0/10]   ack-retries=0   gave-back=1
```

★★★ **`gave-back` == stranded batches, and unacked == gave-back × 10, in both runs.** `:518` is the
**only** producer of `gb-tick = 1`, so that counter has always measured exactly one thing: a
check-ladder abandonment. **Nothing is given back.** The batch is stranded. The instrument was
already present and its *name says the opposite of what happened.*

⚠ And the previous stone fixed the ack ladder — proven live by `ack-retries=5` — which was **not**
the one firing. One shape, hand-rolled three times, is why fixing one instance left the failure
untouched.

## ⛔ THE PROBE — the loop cannot be shared; the outcome can

The obvious fix is one combinator for all three. It is **not available**, and the probe says so:
`wat-scripts/scratch-pad/probe-a-local-fn-can-be-generic.wat`.

Two ladders speak to a `Seen` peer, one to a `Queue` peer, so a shared combinator must be generic
over `[Op Reply]` — and it must be a **local `fn`**, because the worker runs in a process child with
`:env-fn "(:wat::program::EmptyEnv)"` (`:2086`) and cannot see a top-level `defn`. The generic local
form defines cleanly and dies at every call site:

```
(value head): parameter #1 expects :wat::core::T; got :wat::core::i64
(value head): parameter #2 expects [:wat::core::T :-> :wat::core::T];
              got [:wat::core::i64 :-> :wat::core::i64]
```

A local `fn` in a `let` is a monomorphic **value**; its type parameter is never instantiated by
application. (`wat/core.wat:1349` emits that same form, but from inside a macro, where the types are
already concrete.)

★★★★ **So the thing that must stop being re-hand-rollable is not the LOOP — it is the OUTCOME.** A
`bool` retry flag can be dropped, and at two of three sites it *is* dropped. An enum variant cannot
be: the match must name it.

## ⛔ THE ONE CONTRACT DECISION — exhaustion is a variant

```wat
(:wat::core::defenum :fanout::RetryOutcome :- [P R] :wat::enum::Pure
  :Got       [peer <- P  reply <- R]
  :Exhausted [peer <- P  attempts <- :wat::core::i64])
```

Parametric `defenum` is well-precedented — `wat/spawn.wat:195` (three params),
`wat/cache.wat:172`. It is a **type**, so it travels into the process child; only `defn`s do not.

All three ladders return it. No call site can proceed without naming `Exhausted`.

**And the two unfixed ladders get the fix that is already proven:** retry with the inlined backoff
(exponential, jittered, capped) bounded by `vis-ns / 1000000`, exactly as the ack path now does
(`:606`). The rationale is unchanged and holds for all three — after vis expiry the message is
visible again and retrying is pointless.

### `gave-back` is renamed

It counts check-ladder exhaustion and its name asserts the opposite. It is confined to
`circuit.wat` and **asserted in no test**, so the rename is one file and no churn. Each site gets a
counter named for what it measures.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

★ **The gate is binary and currently failing in both our hands:** `2000 4 3 8192 true` completes,
or it does not. Unlike the previous stone, the mechanism is now *measured* rather than inferred —
`gave-back` names the exact event and the exact batch count.

⚠ **This is two rungs, deliberately.** The enum is the rung where the mistake cannot be *expressed*;
the backoff is the fix to the instances that exist. Shipping only the enum would change no
behaviour; shipping only the backoff would leave the shape free to regrow a fourth time.

⚠ **It does not explain the 25 % slope** (4348 → 3244 across n=100→1000). That was measured with
`gave-back=0` throughout, so it has a different cause and stays open.

## OUT OF SCOPE — REJECTED

- **One shared retry loop.** Probed and refused by the type-checker; see above. Three loops, one
  outcome type.
- **De-duplicating the backoff formula.** It now exists in three copies (`:1204` helper, `:1540`
  publisher child, `:675` worker child) with `100` hardcoded in two, because `EmptyEnv` children
  cannot see script helpers. Real, recorded in the tracker, and its own stone — a macro is the
  candidate, since expansion happens before the service form is shipped.
- **Shortening `vis` on normal runs.** Would give a stranded batch a redelivery net, but changes
  what the circuit tests. A different ruling.
- **The receive path's 1000 ms deadline** (`:452`). A receive timeout leaves the message visible,
  which is already safe.
