# Arc 107 — `option::expect` / `result::expect` — INSCRIPTION

## Status

Shipped 2026-04-29 as wat-level helpers in `:wat::std::*`.

## Interim namespace; arc 108 promotes to `:wat::core::*` special forms

Arc 107 ships these as wat-level functions in
`wat/std/option.wat` + `wat/std/result.wat` because that's the
fastest path to closing today's silent-disconnect-cascade hang
in proof_004. The verbs work; the cure is real; the migration
of proof_004's drive-requests is verified.

**The user has named the namespace as wrong.** `expect` belongs
in `:wat::core::*` next to `match` / `if` / `try` — branching
constructs on sum types — not in `:wat::std::*` next to
service-template / stream / hermetic-runner conveniences.

Arc 108 (queued) promotes the helpers to special forms in
`:wat::core::*`:

- `:wat::core::option::expect` and `:wat::core::result::expect`
  registered as special forms (per `:wat::core::try`'s precedent;
  `infer_try` in `check.rs::2225`).
- Syntax: `(:wat::core::option::expect opt -> :T msg)` —
  explicit `-> :T` arm-result type, mirroring `match` and `if`.
- Runtime dispatch lives in `runtime.rs` (a thin wrapper over
  the match + `assertion-failed!` pattern).
- `wat/std/option.wat` and `wat/std/result.wat` retire.
- proof_004 call sites migrate to the new syntax.

The two-arc split is intentional: arc 107 closes the proof_004
deadlock TODAY with the interim shape, and arc 108 lands the
right-namespace special-form variant when there's time for the
substrate dispatch + checker integration. No regression in
between — the `:wat::std::*` defines work; arc 108 deletes them
along with the migration.

## What this arc adds

Two wat-level helpers in `wat/std/`:

```scheme
(:wat::std::option::expect<T>
  (opt :Option<T>) (msg :String) -> :T)

(:wat::std::result::expect<T,E>
  (res :Result<T,E>) (msg :String) -> :T)
```

Both panic with the caller-supplied message when the value is the
failure variant (`:None` / `Err`). On success they return the
inner value. Pure wat composition over `:wat::core::match` +
`:wat::kernel::assertion-failed!`.

Plus one substrate change:

- `:wat::kernel::assertion-failed!` is now `∀T. ... -> :T` in the
  type checker (was `:()`). The runtime never returns from
  assertion-failed!; declaring `:()` was a lie that blocked the
  helpers from typing — they need their `:None` / `Err` arms to
  return `:T`. The polymorphic scheme is the honest one. Existing
  call sites (every `:wat::test::assert-*`) continue to type-check
  unchanged because `T` unifies with their context's `:()`.

## What this arc does NOT add

- Does NOT add a substrate `send!` / `recv!` primitive. The
  expect helpers compose over the existing `:Option<()>`-returning
  primitives — strict semantics is achieved at the call site by
  wrapping `(:wat::kernel::send tx v)` in `option::expect`.
- Does NOT change `:wat::kernel::send` / `recv`. Their
  `:Option<...>` returns are preserved — call sites that handle
  disconnect-as-data (Stream's `for-each`, Service/loop's
  `ack-all`, graceful shutdown patterns) keep working.
- Does NOT migrate `:wat::telemetry::Service/batch-log` to strict.
  The helper's existing contract is "fire and proceed regardless";
  callers who need strict semantics wrap their own batch composition.

## Slice 0a — wat-cli stdin proxy hang (arc 107a)

Surfaced during slice 1 verification. The wat-cli's stdin proxy
thread reads from the cli's real stdin (a tty under interactive
use) and writes to the child's stdin pipe. After the child exits,
the cli's `let _ = stdin_proxy.join()` blocked indefinitely
because the proxy was still in a blocking `libc::read(0, ...)` on
the terminal — even though the child had already gone.

This bit any wat program that exits before consuming stdin: a
panic (like our `expect` helpers), an early return, anything
quick. proof_004's diagnosis (2026-04-29 morning) called this
out — with the helpers ready, an interactive `wat /tmp/test.wat`
that should panic in 50ms instead hung for 2 minutes.

Fix in `crates/wat-cli/src/lib.rs` (line ~276): drop the
stdin_proxy join call. Let the thread die when the process exits.
Keep the stdout_proxy + stderr_proxy joins (those exit naturally
on child-side fd close). Verified:

| Case | Before | After |
|---|---|---|
| `wat /tmp/test-result-err.wat` (Err panic) | hung 2+ min | exits in <50ms with `EXIT=2` and panic message |
| `wat /tmp/test-expect.wat` (Some happy) | exits 0 | exits 0 |
| `wat-cli` test suite (12 tests, includes SIGTERM cascade) | green | green |

## Slice 1 — the helpers + tests

Files added:
- `wat/std/option.wat` — `:wat::std::option::expect<T>` define.
- `wat/std/result.wat` — `:wat::std::result::expect<T,E>` define.
- `src/stdlib.rs` — `WatSource` entries to bake them into the
  binary alongside the rest of the wat stdlib.
- `src/check.rs` — type scheme on `:wat::kernel::assertion-failed!`
  promoted from `:()` to `∀T. ... -> :T`.

Tests verified by direct execution via `wat <file>`:

| File | Result |
|---|---|
| `/tmp/test-expect.wat` (Some 42) | prints `42`, exits 0 |
| `/tmp/test-expect-none.wat` (None) | panic at line:col, message "broker disconnected", exits 2 |
| `/tmp/test-result-expect.wat` (Ok 99) | prints `99`, exits 0 |
| `/tmp/test-result-err.wat` (Err) | panic at line:col, message "expected Ok value", exits 2 |

Panic locations point at the `option::expect` / `result::expect`
call site in user code (per the wat panic_hook's location
machinery — the helpers don't appear in the trace as the failing
frame).

## Slice 2 — proof_004 migration

`holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/004-cache-telemetry.wat`'s
drive-requests now uses `option::expect` at three sites:

```scheme
;; Put fire-and-forget — pre-arc-107: ((_ :Sent) (send ...))
((_ :())
 (:wat::std::option::expect
   (:wat::kernel::send cache-req-tx
     (:wat::holon::lru::HologramCacheService::Request::Put k v))
   "drive-requests Put: cache-req-tx disconnected — cache thread died?"))

;; Get send — same shape, different message
((_ :())
 (:wat::std::option::expect
   (:wat::kernel::send cache-req-tx (Get k reply-tx))
   "drive-requests Get: cache-req-tx disconnected — cache thread died?"))

;; Get reply — recv's outer Option (channel still open) becomes
;; a panic; only the inner Option (Get's "key was present" answer)
;; survives.
((_reply :Option<wat::holon::HolonAST>)
 (:wat::std::option::expect
   (:wat::kernel::recv reply-rx)
   "drive-requests Get: reply-rx disconnected — cache thread didn't reply"))
```

Re-run after migration:

```
running 6 tests
test 004-cache-telemetry.wat                  ... ok (64ms)
test 004-step-A-rundb-alone.wat               ... ok (48ms)
test 004-step-B-cache-alone.wat               ... ok (9ms)
test 004-step-C-both-null-reporter.wat        ... ok (45ms)
test 004-step-D-reporter-never-fires.wat      ... ok (46ms)
test 004-step-E-reporter-fires-once.wat       ... ok (50ms)
test result: ok. 6 passed; 0 failed; finished in 326ms
```

The strict drive-requests is invisible under the happy path.
Under failure (cache thread panics; cache-req-tx disconnects), the
NEXT send panics with the message — no more cascade-into-recv-hang.

## What this arc closes

The "silent disconnect → recv hang" class:

1. Worker thread panics for some reason X.
2. Channel into worker disconnects.
3. Producer's `(:wat::kernel::send tx v)` returns `:None` —
   silently — and the producer keeps going on a dead service.
4. Producer eventually `(:wat::kernel::recv rx)`s on a reply
   channel where the producer itself still owns the Sender (the
   send-then-recv pattern in request/reply protocols).
5. recv blocks forever because the channel isn't disconnected
   from its perspective.
6. The worker's panic message sits unread on the spawn-outcome
   channel; the test hangs.

With `option::expect` at the producer's send sites, step 3 panics
loudly at the call site naming the contract violation. The test
fails fast with a message that points at the disconnected channel,
and the operator's eyes go to the worker for the underlying X.

The proof_004 diagnosis at
`holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DEADLOCK-DIAGNOSIS-2026-04-29.md`
is the worked example of the cascade. Arc 107's helpers are the
defense-in-depth: even if the worker bug recurs, the producer's
strict sends turn it from a hang into an obvious failure.

## The four questions (final answers)

**Obvious?** Yes. The verbs mirror Rust's `Option::expect` /
`Result::expect`. Anyone familiar with either Rust or Clojure
reads the call site and knows what happens.

**Simple?** Yes. ~10 lines of wat each (the helpers). One
type-system change (assertion-failed! polymorphic). One cli fix
(skip stdin_proxy.join). No new substrate primitive.

**Honest?** Yes. The helpers are exactly what they say. The
assertion-failed! polymorphism is more honest than the prior
`:()` declaration (the function never returns). The cli fix is a
minimal correction to a real defect surfaced by today's work.

**Good UX?** Yes. The call site reads:

```scheme
((_ :())
 (:wat::std::option::expect
   (:wat::kernel::send tx v)
   "<contract violation message>"))
```

Intent at the call site; failure mode is loud; the message names
what went wrong. Compared to the prior `((_ :Sent) (send ...))`
that silently absorbed disconnect, the verbosity is honest cost
for a real benefit.

## Cross-references

- `docs/arc/2026/04/107-option-result-expect/DESIGN.md`
- proof_004 diagnosis at
  `holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DEADLOCK-DIAGNOSIS-2026-04-29.md`
- proof_004 stepping stones at
  `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/004-step-*.wat`
- arc 028 — `:wat::core::try` (error-propagation companion).
- arc 064 — `:wat::kernel::assertion-failed!` (the panic primitive
  these helpers compose over).
- arc 095 — `send`'s `:Option<()>` return (the soft-send contract).
- arc 104c — wat-cli's fork architecture and stdio proxy threads
  (the slice-0a fix tweaks this layer).
