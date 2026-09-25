# WEIGH — STONE 255.35: `AcceptOutcome` says what happened — ACCEPTED (vocabulary 2 of 5)

**Executor: grok via pulsare, commit `9589446bc`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T08-52-29Z` | `6122 tests run: 6122 passed`; both `arc255_35` rows PASS (`a_dropped_thread_listener_is_closed`, `a_stop_is_stopped_on_both_loci`) |
| a dropped thread listener | `wat tests/kernel/probe_arc255_35_accept_says_what_happened.wat` | `"thread-dropped Closed"` |
| `AcceptFail::Failed` producers | grep `src/` | only in the socket listener (`listener.rs` ~:424–478): **process only**, as documented |

Taken from the SCORE:

- the stop rows are driven from Rust (wat has no cascade verb) by writing the wake pipe, the way
  `probe_arc278_shutdown_priority_is_the_ruling` does. Pre-stone `Closed`, now `Stopped`, on both loci;
- clippy 0; census `no STOP-8` (215, no rc changed); delta NEW 2 / RECOVERY 0; ledger 208.

## What landed

`AcceptOutcome` has four facts:

| variant | fact | loci |
|---|---|---|
| `Accepted` | a peer was admitted | all |
| `Closed` | every sender dropped; gone for good | thread |
| `Stopped` | a stop was requested; nothing was dropped | all |
| `Failed` | io | process |

- **Process `Closed` is gone:** it was only ever a stop.
- **The thread `Failed` arm is deleted.** The thread receiver can only yield a value, `Disconnected` or
  `Shutdown` (`src/comms/thread.rs:191`), so a `DecodeError` there is a substrate bug and now raises.
- **This time the fact table's nullary shapes matched the arms**, and the SCORE measured it before
  choosing. That is 255.34's lesson applied.
- The 4 consumers got a `Stopped` arm beside `Closed`, with the same dying body. None branched on
  stop-vs-drop.

**Carried:** the copied bodies still say *"listener closed before …"* on the `Stopped` arm. They die
either way, but the sentence misnames a stop. A wording fix; small; for any later stone that touches
those four tests.
