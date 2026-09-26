# WEIGH — STONE 255.43: the telemetry stdlib faces the outcomes it swallows — ACCEPTED (journal); span STOPPED correctly

**Executor: grok via pulsare, commit `2e163e2c1`.** Weighed by the orchestrator against disk on 2026-09-26.

## Re-measured

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-26T00-13-57Z` | `6137 tests run: 6137 passed (10 slow), 22 skipped`; the `arc255_43` row PASSES |
| `_es` in `wat/telemetry/journal.wat` | grep | gone: `ensure-schema` is matched, and every non-`Success` arm fails start-up with its reason |
| `Span::LogResponse` (`wat/telemetry.wat` ~:257) | read | `:Ok` · `:RequestTooLarge` · `:RequestMalformed`: **no arm can carry a journal write failure** |

Taken from the SCORE: pre-stone the journal row **panicked** with *"journal start swallowed the schema failure
and returned Aggregate(…Handle…)"*, and post-stone the owner receives exactly `journal ensure-schema fatal:
SCHEMA-SETUP-FAILED`. Clippy 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0; ledger 198.

## What landed

- **The journal no longer starts over a failed schema.** Its `:init` matches `EnsureSchemaResponse` and fails
  start-up through the startup-crash path (arc 278) with the store's reason.
- **The stdlib `_name` census, classified:**
  - `stdio.wat` `_bytes` is a byte count (the service replies `WriteResponse.Ok`): left;
  - `rete/compile.wat` `_fact-ty`/`_rhs-fence` are evaluated **for their raise, as a fence**: left. ⚠ Under
    the totality ruling, "a raise is the fence" is itself a thing to retire later;
  - `stdio-write-out`/`-err` already face every arm.

## ⛔ STOP-1, correctly taken: `span` cannot tell the truth with its current vocabulary

`wat/telemetry/span.wat` ~:98 still binds `_w` and replies `LogResponse.Ok` whether or not the journal write
succeeded. Mapping a store `Fatal` onto `RequestMalformed` would call a store failure a malformed request. **A
new response variant is a vocabulary ruling.**

**The shape already exists next door:** `Span::CloseResponse` passes the sink's outcome through
(`:Done · :Constraint · :Transient · :Fatal` plus the two wire-breach arms). For the builder:

| option | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **L1: `LogResponse` mirrors `CloseResponse`, gaining `:Constraint [err]` · `:Transient [err]` · `:Fatal [err]` from the journal's write** | Y: the same shape as its sibling | Y | Y: the span reports what the sink said | Y |
| L2: keep `LogResponse`, drop the write's outcome | Y | Y | **N**: it says `Ok` about an unchecked write | — |
| L3: map a failure onto `RequestMalformed` | **N** | — | **N**: a store failure is not a malformed request | — |
