# WEIGH — STONE 255.34: `ConnectOutcome` says what happened — ACCEPTED (vocabulary 1 of 5)

**Executor: grok via pulsare, commit `12e4b302a`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T08-33-51Z` | `6120 tests run: 6120 passed (9 slow), 22 skipped` |
| a dropped thread listener / a dead process listener | `wat tests/kernel/probe_arc255_34_connect_says_what_happened.wat` | `Closed …listener was dropped` · `Closed …Connection refused (os error 111)` (pre-stone both `Refused`) |
| an inert wire copy | `wat …_thread_address_to_process.wat` | `Undialable: …one that crossed a wire is inert.` (pre-stone `Rejected`) |
| `ConnectOutcome.Refused` / `ConnectFail::Refused` / `RETRYABLE` in `src/`, `wat/`, `tests/` | grep | **0** |

Taken from the SCORE:

- `WrongPeer` is driven by a live connect to a wrong minter pid, asserted as the whole sentence;
- the codemod `connect-outcome-facts.wat` has a replay fixture; its dry-run equals its apply, byte for byte;
- the first floor went red 9 times, all this stone's, each fixed at its cause, then the whole floor was
  re-run;
- clippy 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0; ledger 208.

## ⭐ The retryable lie is gone

`Refused` and every copy of *"RETRYABLE … the server may come up"* are removed. `ConnectOutcome` is now five
facts, each documented with the loci that produce it:

| variant | fact | loci |
|---|---|---|
| `Connected` | dialed and admitted | all |
| `Closed` | nothing listening, gone for good | all |
| `Undialable` | an inert wire copy | thread |
| `WrongPeer` | identity mismatch | process |
| `Failed` | io | process |

Measured: an absent autobind name yields `ECONNREFUSED` (os error 111). `ENOENT` does not occur on that
path. A kernel-minted name is never rebound, so on this locus **refused means gone for good**. Fact O
(genuinely retryable) stays reserved for a future TCP locus, per ruling R2.

## Where the brief was wrong, and the code won

- **`Closed` keeps a `[cause <- Failure]` field.** Every consuming arm binds `{:cause c}`, including the
  quasiquoted assemble in `wat/core.wat`. The cause is the producer's description of the connect failure,
  not a death reason. The supervisor ruling (reasons only to the owner) concerns a *death's* reason, so it
  is not engaged. **Carried to the remaining stones:** the fact table's nullary `Closed` came from
  intueri's derivation, and each later stone must measure its consumers' fields before it fixes a variant's
  shape.
- 208 of the 226 `Rejected` arms were `assertion-failed!`. **None read the cause differently by fact**, so
  splitting into `Undialable`/`WrongPeer` with the same body is honest.
