# Arc 107 — `option::expect` / `result::expect` (no silent disconnect)

**Status:** in flight (2026-04-29)
**Predecessor:** arc 095 (paired channels — `:wat::kernel::send`
returns `:Option<()>` and reports disconnect as `:None`).

## The finding driving this arc

proof_004's deadlock diagnosis (2026-04-29) showed how a panicked
worker thread can turn into a hang on a completely different
channel — and the responsibility for the hang lives in the
substrate, not the test.

The cascade:

1. The cache-service worker T2 panicked when its reporter called
   `:wat::holon::Atom` on a Struct value (a real bug in
   `holon-lab-trading/wat/cache/reporter.wat`).
2. T2's stack unwound; cache-req-tx's underlying channel
   disconnected.
3. The test thread's subsequent
   `((_p :wat::kernel::Sent) (:wat::kernel::send cache-req-tx ...))`
   returned `:None` (the documented disconnect signal). The test
   bound it to `_p` and continued.
4. The test reached a Get and sent
   `(Get k reply-tx)` — also returned `:None`. Crucially, the
   `reply-tx` was NOT moved into the channel (the send failed),
   so it remains alive in the test's let* binding.
5. The test then `(:wat::kernel::recv reply-rx)`. The reply
   channel is NOT disconnected — the test itself owns the only
   live Sender. recv blocks forever.
6. Foldl never completes; `_cache-join` is never reached; T2's
   panic sits unread on the spawn-outcome channel; the test
   hangs with no CPU.

Diagnosis archived at
`holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DEADLOCK-DIAGNOSIS-2026-04-29.md`.

## The user's framing

> "i'm very tired of dealing with deadlocks"
> "we have try -- do we need to rename try?"
> "so both try and expect should co-exist?"

The exchange clarified two things:

1. `:wat::core::try` is the **error-propagation** form (Rust's `?`).
   It punts `Err(e)` UP the call stack as the enclosing fn's
   return. Compile-time-checked: enclosing fn must return
   `Result<_, E>`. It already exists; nothing to rename.

2. What today's deadlock-cure needs is the **panic** form (Rust's
   `Option::expect` / `Result::expect`). On `:None` / `Err`, panic
   with a message. Works in any context — does not need an
   enclosing Result return type. This is genuinely new ground.

The two verbs are complementary and live side by side. `try` is
for protocols that handle failure as data; `expect` is for sites
where a missing value means a substrate bug (test scaffolding,
contract-strict producers, single-consumer pipelines whose
consumer's death IS the bug to surface).

## The cure — wat-level helpers

Two wat-level functions, each ~5 lines of wat over existing
substrate primitives. NO new substrate primitive. NO change to
existing `send` / `recv` semantics.

```scheme
;; wat-rs/wat/std/option.wat
(:wat::core::define
  (:wat::std::option::expect<T>
    (opt :Option<T>)
    (msg :String)
    -> :T)
  (:wat::core::match opt -> :T
    ((Some v) v)
    (:None
      (:wat::kernel::assertion-failed! msg :None :None))))

;; wat-rs/wat/std/result.wat
(:wat::core::define
  (:wat::std::result::expect<T,E>
    (res :Result<T,E>)
    (msg :String)
    -> :T)
  (:wat::core::match res -> :T
    ((Ok v) v)
    ((Err _e)
      (:wat::kernel::assertion-failed! msg :None :None))))
```

That's the entire substrate-side change. Two files, ~10 lines of
wat each (counting docstring + signature). The `assertion-failed!`
primitive already exists (arc 064 + arc 088 lineage); these
helpers are pure composition.

## When to use which

| Verb | Failure case | Where |
|---|---|---|
| `:wat::core::try` | `Err(e)` propagates UP | Inside a fn returning `:Result<_, E>` |
| `:wat::std::option::expect` | `:None` panics with message | Anywhere |
| `:wat::std::result::expect` | `Err(_)` panics with message | Anywhere |

Mirrors Rust:
- `?` (try-equivalent) — propagation
- `.expect("msg")` — panic
- Both ship in `std`; both are common; the user picks per call site.

## Stepping stones

### Slice 1 — the helpers + tests

- `wat-rs/wat/std/option.wat` — `:wat::std::option::expect<T>` define.
- `wat-rs/wat/std/result.wat` — `:wat::std::result::expect<T,E>` define.
- `wat-rs/wat-tests/std/option.wat` — test happy path (Some
  returns inner) and panic path (None panics with the message).
- `wat-rs/wat-tests/std/result.wat` — test Ok returns inner; Err
  panics.

### Slice 2 — proof_004 migration

The lab's drive-requests + reporter's batch-log all discard
`:Option<()>` from `send` calls. After this slice, every such
site uses `option::expect` so the FIRST disconnect lands at the
call-site as a clear panic, instead of cascading into a
recv-hang.

Specific call sites:
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/004-cache-telemetry.wat`
  drive-requests — the 30-iteration foldl's `((_p :Sent) (send ...))`.
- `holon-lab-trading/wat/cache/reporter.wat` — currently calls
  `Service/batch-log`. The reporter is on the cache thread; if
  rundb dies, the reporter should panic loudly so the cache
  thread's panic message names the actual problem. Wrap the
  inner `send` / `recv` with `option::expect`.

After slice 2 the six deftests in proof_004's directory still
pass green:

| Test | Result |
|---|---|
| 004-cache-telemetry | ok (full proof) |
| 004-step-A-rundb-alone | ok |
| 004-step-B-cache-alone | ok |
| 004-step-C-both-null-reporter | ok |
| 004-step-D-reporter-never-fires | ok |
| 004-step-E-reporter-fires-once | ok |

### Slice 3 — INSCRIPTION + USER-GUIDE + 058 row

- `wat-rs/docs/arc/2026/04/107-*/INSCRIPTION.md` — capture the
  cure and cross-references.
- `wat-rs/docs/USER-GUIDE.md` — short paragraph on
  try-vs-expect-vs-soft-Option, with the table from this DESIGN.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — one-line row.

## What this does NOT do

- Does NOT add a substrate primitive. Both helpers are pure wat
  over `match` + `assertion-failed!`.
- Does NOT change `:wat::kernel::send` / `recv`. Their
  `:Option<...>` returns are preserved — call sites that handle
  disconnect-as-data (Stream's `for-each`, Service/loop's
  `ack-all`, graceful shutdown patterns) keep working.
- Does NOT extend `:wat::core::try` to `:Option<T>`. That's a
  reasonable follow-up arc; today's bug doesn't need it.
- Does NOT promote `Service/batch-log` to strict by default.
  The helper's existing contract is "fire and proceed regardless";
  callers who need strict semantics wrap their own batch-log
  composition. No surprise breakage in the rest of the substrate.

## The four questions (answered)

**Obvious?** Yes. Two verbs that mirror Rust's `Option::expect`
and `Result::expect` with the wat naming convention. Reader
knows what they do without consulting docs.

**Simple?** Yes. ~10 lines of wat each, no Rust, no checker
change. The helpers are pure composition over existing primitives.

**Honest?** Yes. The helpers are exactly what they say — match
the sum-type, return the success case, panic-with-message on the
failure case. No hidden semantics. The DESIGN names what they
DO solve (silent-disconnect cascades, missing-value bugs) and
what they DON'T (the underlying panic in the worker still has
its own root cause to fix).

**Good UX?** Yes. Caller site reads:
```scheme
((_ :()) (:wat::std::option::expect (:wat::kernel::send tx msg)
            "send to broker disconnect — broker died unexpectedly"))
```
Intent at the call site; failure mode is loud; the message names
what went wrong. Compare to the current pattern of binding
`_send :wat::kernel::Sent` and silently discarding. The cost is
verbosity; the benefit is the silent-disconnect-deadlock class
above does not happen.

## Cross-references

- `feedback_silent_disconnect_hang.md` (user memory) — the
  pattern this arc defends against.
- `feedback_iterative_complexity.md` (user memory) — the
  stepping-stone discipline that produced today's diagnosis.
- proof_004 stepping stones at
  `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/004-step-*.wat`
  — the audit trail.
- arc 095 — `send`'s `:Option<()>` return.
- arc 028 — `:wat::core::try` (error-propagation Result form).
- arc 064 — `assertion-failed!` (the panic primitive these
  helpers compose over).
