# BRIEF — the death notice is not a malformed frame

Turn both remaining floor reds green. `src/runtime.rs` + `wat/service.wat` + four call sites.

Read `DESIGN-the-death-notice-is-not-a-malformed-frame.md` first — **including its correction of
my previous exemplar citation.**

## ⚠ THE MIS-CITATION YOU FOLLOWED CORRECTLY

The last brief sent you to `poll:27194` (`ServiceEvent::Malformed`) for select's EDN-decode arm.
**That was my error, not yours.** `poll`'s `Malformed` is a live client's junk message. A **peer**
whose frame will not decode is a dead peer, and `recv:25047` is the right exemplar. Change that
arm; **keep the other eight conversions you made.**

## READ IN ORDER

| room | why |
|---|---|
| `src/kernel/peer.rs:460-485` | the reserved keyword sentinels a dying peer sends. **This is what select could not decode** |
| `src/kernel/peer.rs:432`, `:443` | `is_peer_crashed_sentinel` / `is_peer_severed_sentinel` — **the exemplar for change 2** |
| `src/runtime.rs:25047` | `recv`'s decode arm returns `Lost`. **The exemplar for change 1** |
| `src/runtime.rs:25124-25164` | `recv` mapping `RecvError::PeerCrashed` / `PeerSevered` to `Panic` / `Severed` |
| `src/runtime.rs:6541-6560` | the arc-294 scrub — unchanged, and it must stay intact |
| `wat/service.wat` `CallOutcome` and its four circuit call sites | change 3 |

## THE THREE CHANGES

1. **process tier** — select's EDN-decode and UTF-8 arms return `ServiceEvent::Lost` (was
   `Malformed`), matching `recv:25047`.
2. **thread tier** — select checks the two sentinel predicates before treating a `Value` as a
   message, and returns `Lost` carrying the matching cause.
3. **`CallOutcome`** — `PeerGone` splits into `Lost [cause]` and `Closed`; the generated method
   maps `Lost` to `RecvOutcome::Lost(cause)` instead of flattening to `Closed`. Update the four
   circuit call sites, which today treat both identically.

## BLAST RADIUS

`src/runtime.rs`, `wat/service.wat`, and the four `call-by-deadline` sites in
`wat-scripts/fanout/circuit.wat`. **No codemod** — `CallOutcome` has four callers, not 600.

⚠ A `wat/` edit is frozen at build time — rebuild before every run.

## STOP TRIGGERS

- **STOP-1** — if the process tier cannot distinguish a **severed** peer from any other
  undecodable frame, STOP and report. `Severed` may be thread-tier-only; inventing a
  process-tier sever would be fabricating a cause.
- **STOP-2** — if any newly returned cause carries text beyond the canonical reason-free
  `message_only_failure`, STOP. Arc 294 stands: a client learns no server internals.
- **STOP-3** — if splitting `PeerGone` needs more than the four known call sites, STOP and
  report the count before editing.
- **STOP-4** — do not touch `poll:27124` (admin channel), the rung-3 migration, or perf.

## PRIOR RESULT TO COPY

`SCORE-select-returns-what-it-sees.md` — your own last strike. Keep its conversions; change only
the arm my mis-citation aimed wrong.
