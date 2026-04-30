# Arc 105 — `spawn-program` error-as-data + substrate `Vec<String>` retired — INSCRIPTION

**Status:** shipped 2026-04-29 (all four slices same day).

**Predecessor:** [arc 103b's deferral note](../103-kernel-spawn/INSCRIPTION.md#slice-103b--partial-iowriter-close-shipped-sandbox-wat-scaffolded).
Arc 103a shipped `spawn-program`. Arc 103b documented two specific
blockers preventing `wat/std/sandbox.wat` from replacing the
substrate Rust `eval_kernel_run_sandboxed*` impls. Arc 105 closes
both, then deletes the substrate impls.

**Surfaced by:** the same arc-103 conversation. The user direction
("for real work we use real kernel pipes as the surface area of
our programs") drove arcs 103a and 103c to live up to it for new
code; arc 103b deferred the existing substrate's `Vec<String>`
absorbance because two substrate-level changes were prerequisites:

1. spawn-program raises on startup failure → wat-level helper
   can't capture the failure.
2. ThreadDiedError variants don't pattern-match cleanly from wat
   → wat-level helper can't extract panic / runtime messages.

Arc 105 ships those two changes. Then bundles `sandbox.wat` and
deletes the substrate Rust impls.

---

## What shipped

### Slice 105a — `:wat::kernel::StartupError` + spawn-program returns Result

**`:wat::kernel::StartupError`** — new struct in `src/types.rs`:

```scheme
(:wat::core::struct :wat::kernel::StartupError
  (message :String))
```

Single field. Distinct type identity (a wat value of StartupError
can't be confused with random String). Auto-generated
`/new` + `/message` accessor via register_struct_methods.

**spawn-program signature change:**

```scheme
;; arc 103a:
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::Process)            ;; raised on startup failure

;; arc 105a:
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :Result<:wat::kernel::Process, :wat::kernel::StartupError>)
                                        ;; failure-as-data
```

Same change for `spawn-program-ast`. `src/spawn.rs` adds
`startup_error_result` helper that builds
`Value::Result(Err(...))` for both freeze failures and signature
failures. The dispatch arms never raise `RuntimeError` for these
cases now; only for genuine substrate-level argument errors
(arity / type mismatch).

**Caller migrations** (small surface):

- `tests/wat_arc103_spawn_program.rs` — 6 tests. Each test's
  `:user::main` now returns `:Result<X, :wat::kernel::StartupError>`;
  spawn calls wrap in `:wat::core::try`; success cases return
  `(Ok ...)`; Rust `unwrap_ok` helper strips the outer Result
  before existing `unwrap_some_string` / `unwrap_unit` /
  `unwrap_none`.
- `wat-scripts/dispatch.wat` — `:demo::dispatch::run` hoisted to
  `-> :Result<(), :wat::kernel::StartupError>`; uses `try` for
  the spawn. `:user::main` pattern-matches and writes
  `StartupError.message` to stderr on Err.
- `wat-scripts/ping-pong.wat` — pattern-matches inline; panics
  with the StartupError message on Err. (Demo discipline:
  `:user::main`'s fixed `-> :()` signature can't propagate via
  `try`.)

### Slice 105b — `:wat::kernel::ThreadDiedError/message` accessor

```scheme
(:wat::kernel::ThreadDiedError/message
  (err :wat::kernel::ThreadDiedError)
  -> :String)
```

Extracts the carried message string from any variant. Returns
`"channel disconnected"` for the unit variant. Routes around the
wat-side enum-variant pattern-matcher gap arc 103b surfaced — wat
callers can ask for a generic message without discriminating
variants.

### Slice 105c — bundle `wat/std/sandbox.wat`; delete substrate impls

The payoff. With slices 105a and 105b in hand, plus one more
substrate addition (`/to-failure`, see below), `wat/std/sandbox.wat`
becomes the canonical implementation of `:wat::kernel::run-sandboxed`
and `-ast`. The substrate Rust impls in `src/sandbox.rs` delete.
**`Vec<String>` exits the kernel boundary; it survives only inside
the wat-level test-convenience helper where collected output IS the
test assertion target — the discipline arc 103 named.**

**The arc 064 preservation work** — losing the structured
`actual` / `expected` / `location` / `frames` propagation through
run-sandboxed would have been a real regression. Instead of accepting
the regression, slice 105c:

- Widens `SpawnOutcome::Panic` from `(String)` to
  `{ message: String, assertion: Option<AssertionPayload> }`. The
  `catch_unwind` site downcasts to `AssertionPayload` (taking
  ownership) and preserves both message AND structured fields.
- Widens `:wat::kernel::ThreadDiedError::Panic` enum variant to
  carry a 2nd field `failure: Option<:wat::kernel::Failure>`.
  When the panic was an assertion, the structured Failure rides
  through `join-result`; otherwise `:None`.
- Adds `:wat::kernel::ThreadDiedError/to-failure -> :Failure`
  accessor that always returns a structured Failure regardless of
  variant. `Panic` with assertion → use the carried Failure.
  `Panic` without → message-only Failure. `RuntimeError` →
  message-only. `ChannelDisconnected` → `"channel disconnected"`
  message. `wat/std/sandbox.wat`'s `failure-from-thread-died`
  calls this once; no per-variant pattern matching needed.

`src/runtime.rs` gained `extract_panic_payload` (owning sibling of
`format_panic_payload`) plus helpers `value_from_span` /
`value_from_frame_info` / `failure_value_from_assertion_payload` /
`message_only_failure` to build the wat-shape Failure values.

**What deletes:**

- `src/sandbox.rs` shrinks from 723 lines to 124. Gone:
  `eval_kernel_run_sandboxed`, `eval_kernel_run_sandboxed_ast`,
  `build_failure`, `failure_from_*`, `build_run_result`,
  `bytes_to_lines`, `expect_vec_string`, `vec_string_value`.
  Stays: `resolve_sandbox_loader` (called by spawn.rs) + its 3
  unit tests.
- `src/runtime.rs`: `:wat::kernel::run-sandboxed` /
  `-ast` dispatch arms gone. Wat-level `(define ...)` forms in
  `wat/std/sandbox.wat` register their schemes at startup.
- `src/check.rs`: those primitives' substrate-side schemes gone.
- `src/stdlib.rs`: `wat/std/sandbox.wat` BUNDLED (was
  intentionally-not-bundled scaffold from arc 103b).

One pre-existing test pattern updated:
`runtime::tests::join_result_captures_panic_as_data` —
pattern `((Err (:wat::kernel::ThreadDiedError::Panic msg)))` becomes
`((Err (:wat::kernel::ThreadDiedError::Panic msg _failure)))` for
the widened 2-field variant.

### Slice 105d — INSCRIPTION + 058 row + USER-GUIDE update

This file. Plus:

- USER-GUIDE.md §13 (testing) note that
  `:wat::kernel::run-sandboxed` is now a wat-level define in
  `wat/std/sandbox.wat`, not a substrate primitive.
- 058 changelog row.

---

## The honest tradeoff that drove slice 105c's design

Mid-implementation, the substrate-shrinkage path surfaced a
behavioral question: should arc 064's structured `actual`/`expected`
preservation through `run-sandboxed` survive, or accept the
regression?

The four-question discipline answered it:

| | Option A (regression) | Option B (preserve) |
|---|---|---|
| **Obvious?** | No (silent feature loss) | Yes (preserve the promise) |
| **Simple?** | Yes (1 test edit) | More work (~140 LOC + variant widening) |
| **Honest?** | **No** (arc 064 docs lie) | Yes |
| **Good UX?** | **No** (degraded diagnostics) | Yes |

Option A failed two questions; option B failed only on "simple,"
which loses to "honest" + "UX" when the regression is real. Going
with option B preserved the feature and made arc 105 a strict
superset of what the substrate did.

This is the principle the arc preserves: **substrate cleanup is not
a license for silent regressions.** When deleting substrate code in
favor of wat-level reimplementations, the wat-level path either
matches the deleted behavior or the cleanup is incomplete. Arc 105c
chose match-the-behavior; the substrate addition (widening Panic
variant + new accessor) was the cost.

---

## What's now true

After arc 105 ships:

- `Vec<String>` no longer flows through any substrate kernel
  boundary. `:user::main`'s args, spawn-program's pipes,
  fork-program's pipes — all kernel-pipe-typed.
  `Vec<String>` survives only as the assertion target inside
  `wat/std/sandbox.wat`, where the test discipline puts it.
- `spawn-program` and `spawn-program-ast` return Result. Failure
  is data, not a raised exception.
- `ThreadDiedError::Panic` carries structured assertion data when
  available — arc 064's promise propagates through run-sandboxed
  unchanged.
- `wat/std/sandbox.wat` is the canonical run-sandboxed
  implementation. Wat-level. Bundled. Replacing 600 LOC of
  substrate Rust with ~140 LOC of wat (the test helper) + ~140
  LOC of substrate type-conversion helpers.
- The arc 103-104-105 architecture is complete. Real kernel pipes
  for real work; failure as data; substrate shrinks; cli always
  forks; the hologram is geometric.

---

## Lessons captured

1. **Substrate cleanup is not a license for silent regressions.**
   When deleting substrate impls, audit the features the deleted
   impl shipped. Arc 064's structured-assertion preservation was
   one such feature; the cleanup arc had to match the behavior or
   the deletion was incomplete.

2. **The four-question discipline is decisive.** "Obvious / simple
   / honest / good UX" gave clear separation between the easy-but-
   dishonest path and the right path. Simple lost to honest + UX
   when the regression was real.

3. **Variant widening is cheap when the codebase routes through
   accessors.** ThreadDiedError::Panic widening from 1 to 2 fields
   broke exactly one test (which used direct variant
   destructuring); every other consumer routes through `/message`
   or `/to-failure` accessors. Schema flexibility comes from
   accessor-based access, not pattern-match-based access.

4. **Owning extraction is what `Box<dyn Any>` wants.** The deleted
   substrate `failure_from_panic_payload` did
   `payload.downcast::<T>()` to take ownership of the
   AssertionPayload's owned String fields. Arc 105c restored this
   via `extract_panic_payload(Box<dyn Any>)` returning
   `(String, Option<AssertionPayload>)`. The borrowing
   `format_panic_payload(&Box<dyn Any>)` survives for non-
   structured paths (e.g., `:wat::kernel::join`).

5. **Two-field variants don't break wat callers that use
   wildcards.** Existing wat code that did `((Err _))` against
   ThreadDiedError stayed working through the variant widening
   because `_` matches any shape. The single test that used
   `((Err (:Panic msg)))` got `_failure` appended for the new
   field. Surgical migration.

---

## Status of the run-sandboxed family

Final state across all relevant arcs:

| Surface | Pre-arc-103 | After arc 105 |
|---|---|---|
| `:wat::kernel::run-sandboxed` | substrate Rust | wat-level (`wat/std/sandbox.wat`) |
| `:wat::kernel::run-sandboxed-ast` | substrate Rust | wat-level (same file) |
| `:wat::kernel::run-sandboxed-hermetic-ast` | substrate Rust (retired earlier) | wat-level (`wat/std/hermetic.wat`, atop `fork-program-ast`) |
| `:wat::kernel::spawn-program` | (didn't exist) | substrate Rust, returns `:Result<:Process, :StartupError>` |
| `:wat::kernel::spawn-program-ast` | (didn't exist) | substrate Rust, same shape |
| `:wat::kernel::fork-program` | (didn't exist) | substrate Rust |
| `:wat::kernel::fork-program-ast` | substrate Rust (was `fork-with-forms`) | substrate Rust, renamed in arc 104a |
| `:wat::kernel::ThreadDiedError/message` | (didn't exist) | substrate Rust accessor (arc 105b) |
| `:wat::kernel::ThreadDiedError/to-failure` | (didn't exist) | substrate Rust accessor (arc 105c) |
| `:wat::kernel::StartupError` struct | (didn't exist) | substrate type (arc 105a) |

The substrate keeps the load-bearing primitives (spawn / fork /
pipe / join / etc.). The convenience helpers (run-sandboxed
collecting output to Vec<String>) live at the wat layer where they
belong. Vec<String> exits the kernel.
