# FINDING — INTUERI names the two `recv` vantages (W2)

**Cast 2026-09-25**, read-only, at HEAD `ac39cc7ce`. The spell was embedded verbatim. The builder ruled W2
(two outcome types, one per vantage) with four YES, and *"intueri names all"*. The orchestrator verified C1
on disk.

## The names (four questions per candidate; the survivor's row is all YES)

| thing | survivor | rejected, with the NO |
|---|---|---|
| the peer outcome type | **`PeerRecvOutcome`** | `RecvOutcome` (it claims to be *the* outcome of `recv`), `PeerOutcome` (send and connect have peer outcomes too), `PeerReadOutcome` (the verb is `recv`) |
| the owner outcome type | **`OwnerRecvOutcome`** | `SupervisorRecvOutcome` (long; most holders are not supervising), `SpawnedRecvOutcome`/`ChildRecvOutcome` (they read as the child's own vantage) |
| the peer's reasonless crash notice | **`Crashed`** (nullary) | `Died` (the same name as the owner's but without a reason), `PeerCrashed` (repeats the type), `ServiceCrashed` (ties a general read to one role) |
| the owner's reasoned death | **`Died [cause <- LociDiedError]`** | `Lost` (a Level-1 lie), `Crashed [cause]` (`LociDiedError` also carries StartupError and BadReturn, which are not crashes, and it collides with the peer's) |
| the far end refusing my request (`Reply::Failed`) | **`Rejected [cause]`**, **conditional**: `ServiceEvent.Rejected` (an over-budget frame) must become `Oversized` first or in the same pass | `Failed` (collides with io), `Refused` (reserved, and retired as a lie), `ReplyFailed` (welds the name to the wire enum) |

**Shared variants:** `Closed`, `Stopped`, `HandleClosed`, `Failed`, `Malformed`, `Oversized` are the same
fact under the same name on both types. `Closed` stays nullary on `recv`, because every arm binds `{}`.

## Verified fact sets

| fact | `PeerRecvOutcome` | `OwnerRecvOutcome` |
|---|---|---|
| `Message` | all loci | all loci |
| `Closed` | all loci | all loci |
| `Crashed` | all loci | — |
| `Died [LociDiedError]` | — | all loci |
| `Stopped` | all loci | all loci |
| `HandleClosed` | all loci | all loci |
| `Failed [cause]` | process | process |
| `Malformed [cause]` | process | process |
| `Oversized [cause]` | process | process |
| `Rejected [cause]` | all loci | — |

**C2:** the owner has **more** facts than the brief listed (`Failed`, `Malformed`, `Oversized` on
process). All of them fold into `Lost` today, and `tests/process/probe_arc278_recv_over_budget_reason.wat`
exercises the over-budget one.

## ⛔ Contradictions the ruling must settle first

- **C1 (verified): the vantages cannot be told apart statically.** `wat/spawn.wat:267-268`:
  `(derive :wat::kernel::Thread :wat::kernel::Peer)`, and the same for `Process`. Owner handles are then
  *typed as `Peer`*:
  - `Launched.handle <- (Peer :- [Sh Lu])` (`spawn.wat:329`);
  - every defservice `Handle.handle`;
  - `recv-all-loop`;
  - `Locus/spawn-runner`'s return.

  The census has 8 `recv` sites that are owner at runtime but `Peer` statically, including
  `service.wat:2212/2250/2296/2340` (`recv (Handle/handle h)`), **the stop path that most needs the
  reason**. A checker that picks the outcome type from the receiver type would either leak the reason to
  `Peer`-typed code or drop it at the supervisor. The owner family already exists: `:wat::spawn::Spawned`,
  *"the owner-side spawn-handle marker"* (`spawn.wat:254-258`).
- **C3:** `comms::RecvError::Failed(String)` carries io errors, invalid UTF-8, wire decode failures and
  `FrameScan::Malformed` alike. A `RecvError::Malformed` must be split out before `Failed`/`Malformed`
  can be reported honestly on either vantage.
- **C4:** `classify_peer_error` maps a crash-channel io failure after output EOF to `PeerDeath::Closed`, a
  clean exit, while `classify_peer_death` maps the same case to `Lost`. **The two classifiers disagree.**
- **A collision outside this cast:** `CloseOutcome.Closed [exit]` (I closed and reaped it) against the
  proposed `OwnerRecvOutcome.Closed` (the child left). Two facts, one name, on the same handle.

## Vantage split, measured (a heuristic static census)

`recv` sites: **267** (tracked `.wat`).

| vantage | sites |
|---|---|
| owner, statically `Thread`/`Process` | 177 |
| owner at runtime, statically `Peer` (C1) | 8 |
| peer | 82 |

About 485 arms per `RecvOutcome` variant; 197 of the `Lost` arms are `assertion-failed!`.

## Rust names (for the stone)

- `PEER_/OWNER_RECV_OUTCOME_TYPE`;
- builders `peer_recv_outcome_*` / `owner_recv_outcome_*`;
- `spawn::PeerRecvError` (owner-side, misnamed) → `OwnerRecvError`;
- `PeerDeath` (holds a "nothing died" `Shutdown`) → `ChildExit`;
- 42 retired-prime `*_prime` fn names across 14 verbs (the prime-comment branch may already cover these;
  `*_readln_prime` is legitimate).
