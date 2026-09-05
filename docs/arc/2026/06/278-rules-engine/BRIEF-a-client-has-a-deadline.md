# BRIEF — a client has a deadline

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`744be9995`, tree clean. Read `DESIGN-a-client-has-a-deadline.md` first.

## THE WORK

Every client call in the circuit waits forever for a reply that may never come. Give `Seen/claim` a
deadline: race the reply against a timer, and on expiry **discard the peer, redial, retry**. The
mechanism is proven and needs no substrate change — you are moving a green probe into the circuit.

## ROOMS

1. **`wat-scripts/scratch-pad/probe-client-deadline-via-select.wat`** — **run it first.** `3/3`,
   `fast=reply; never=TIMED-OUT`. It contains the whole shape: the laundering, the tier match, the
   select, and a server that goes silent. Copy it.
2. **`wat/service.wat:1610-1623`** — the laundering the probe copies, and the comment that names it
   as *"a check-time-only detour"* because type params are erased at runtime.
3. **`wat-scripts/fanout/circuit.wat`** — `:fanout::worker`'s `-tick`, the `Seen/claim` call and its
   `Lost` / `Closed` arms (which already redial). Your timeout path joins them.
4. **`docs/arc/2026/06/278-rules-engine/SCORE-a-peer-is-dead-only-when-redial-fails.md`** — the
   discard-and-redial shape already in the tree. A timeout is a third way to reach it.

## SKETCH

```wat
;; tier = the RACED peer's, not the caller's
(:wat::kernel::after <peer-kind-of-the-connection> (:wat::time::Milliseconds ms) <a Reply value>)
;; laundered through (first (conj (Vector :- [(Peer :- [Op Reply])]) timer))

(:wat::core::match (:wat::kernel::select [peer timer])
  ((:wat::spawn::ServiceEvent::Message idx m) <idx 0 = reply · idx 1 = TIMEOUT>)
  ;; and Closed / Lost / Shutdown / Admin / Connection / Malformed / Rejected — NAME EACH
  )
;; on TIMEOUT: discard the peer, redial the address, retry — bounded, and REPORT on exhaustion
```

## STOP TRIGGERS

1. **The deadline can fire while the server is behaving correctly.** Then it is a fault injector, not
   a fix. `Seen/claim` has no server-side wait, so any positive deadline is honest — if you find one
   that is not, STOP.
2. **You are about to adopt this on `Queue/receive`.** It carries `:UpTo`; its deadline must be
   `UpTo + slack`. S35, its own decision. STOP.
3. **The retry reuses the discarded peer.** That is an infinite loop that looks like a hang. STOP.
4. **You are about to wildcard the `ServiceEvent` arms.** The probe did; a shipped client face does
   not. STOP and name them.
5. **You are about to change `recv`, `select`, or anything in `src/`.** No substrate change. STOP.
6. **The circuit's invariant moves.** `distinct=8000; dup=0` — a finding, not something to tune.

## HOW TO WORK

Foreground everything. Floor is `scripts/floor.sh`; **Summary line, never a piped exit code.** Floor
**after** the edit. On an unintended red: **do NOT re-run**, capture whole, name the arm.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form, arc-109 NOTE.

Leave your work uncommitted. Prior comparable: `SCORE-the-reactor-grows-a-seam-v3.md`.

## REPORT

- the probe, re-run
- **the retry landing on a FRESH peer** — show it, do not assert it
- **`seen-dups`, five runs.** ⛔ Any value is the result. Non-zero means a retry produced a duplicate
  and the consumer absorbed it — the thing chased since R69. Zero means the deadline never fired in
  normal operation, which is also correct
- the circuit: five runs, `total`/`distinct`/`dup`
- the floor Summary line
- every `ServiceEvent` arm you named, and what each does
- every STOP that fired
- **the honest deltas.** Ten of my counts have missed this campaign, and the last three stones each
  found a citation of mine stale. What you find is the fact.
