# WEIGH — STONE 255.36: `SendOutcome` says what happened — ACCEPTED (vocabulary 3 of 5)

**Executor: grok via pulsare, commit `3f3d753ad`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T09-36-33Z` | `6124 tests run: 6124 passed (10 slow), 22 skipped` |
| a far end that left, thread / process | `wat tests/kernel/probe_arc255_36_send_says_what_happened.wat` | `Closed the far end is gone` on both (pre-stone `Lost disconnected`) |
| **a double migration** (the first codemod was not idempotent; a second run renamed the new `Closed` arms to `HandleClosed`) | `HandleClosed {:cause` arms in the corpus | **0**. `Closed {:cause}` arms: 206. `HandleClosed {}` arms: 206. The corpus was migrated once, correctly |
| the IDE's dead-code warnings (`try_send_outcome_failed`, `loci_died_disconnected`) | grep HEAD `src/` | **stale**: neither function exists; the tree is clean |

Taken from the SCORE: clippy 0; census `no STOP-8` (215); delta NEW 2 / RECOVERY 0; ledger 208; the
replay-gate red was caught by the floor, the fixture regenerated, and the floor re-run as a new run.

## What landed

`SendOutcome` (and `TrySendOutcome`, which keeps `WouldBlock`):

| variant | fact | loci |
|---|---|---|
| `Sent` | delivered | all |
| `HandleClosed` | **this** handle was already closed | all |
| `Closed [cause <- Failure]` | the far end is gone for good; the cause is a description of the departure | all |
| `Stopped` | a stop while parked | process only |
| `Failed [cause]` | io | process only |

- **No `LociDiedError` rides a send outcome**, which is the supervisor ruling.
- `Closed` keeps a field because all 203 `Lost` arms bound one (measured; the brief's "about 56 read it" was
  stale).
- The six `service.wat` comments that read the old `Closed` as "client gone" now say what it was: *this
  handle was already closed*. The bodies were unchanged in behaviour.

## Findings, carried

- ⚠ **A non-idempotent codemod was caught by its own replay gate** (`every_recorded_migration_replays_shard_12`,
  `result != after.post`). The fixed script renames a `Closed` arm only when its binder is exactly `{}`.
  **The replay gate did its job:** a codemod that is not idempotent cannot be recorded silently.
- `TrySendOutcome.Failed` is in the `defenum` for exhaustiveness, and **no locus builds it**, since
  `TrySendError` is only `Full`/`Disconnected`. Intueri's rule: a variant with no producer is a reader's
  question. **The next vocabulary stone should rule** whether an unbuilt variant stays for exhaustiveness
  or is dropped (as ruling R2 did for `Refused`).
