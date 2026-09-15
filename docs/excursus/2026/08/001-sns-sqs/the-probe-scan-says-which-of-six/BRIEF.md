# BRIEF — the probe scan says which of six (nine, measured)

**Read `DESIGN.md` beside this first.** It carries the one contract decision, the **nine** worlds this
one message covers, why `Stopped` is reported rather than fixed, why this is deliberately **not** a
codemod, and five trap-doors.

⛔ **Whether these arms raise is UNCHANGED**, with one exception that is **reported, not fixed**.

## The work, in one paragraph

One fold in `:queue::queue` (`wat-scripts/queue/sqs.wat:586`–`:615`) counts landed rows by probe scan,
and every failure path raises the identical string *"queue: probe scan failed — peer is dead, not a
broken pipe"*. Five `RecvOutcome` arms say it, and a `_` over `ScanResponse` says it for four more
variants — **nine worlds, one message, seven of them false.** Name each world, carry the cause where the
variant has one, keep the site label, and change no disposition.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/queue/sqs.wat:586`–`:615` | the fold. `:591` handles `ScanResponse::Success`; `:593` is the `_` that hides four variants; `:596`/`:600`/`:604`/`:608`/`:612` are the five `RecvOutcome` arms. |
| ⭐ `wat/query.wat:548`–`:554` `ScanResponse` | the four variants the `_` hides: **`Transient`** (retryable, carries `err`), `Fatal` (carries `err`), `RequestTooLarge` (`bytes cap`), `RequestMalformed` (`path expected got`). Two of those are **our** defect, not the store's. |
| ⛔ `docs/excursus/2026/08/001-sns-sqs/transient-means-try-again/` | the **struck** stone that made the queue *retry* a transient store error instead of dying on it. This site dies on one. ⚠ Read it before deciding what `Transient`'s message should say — you are **not** making it retry here, but the message must not pretend a retryable error is a death. |
| ⭐ `wat/service.wat` `:wat::service::redial-failed!` (the previous stone) | the shape for *naming a variant + keeping a site label + carrying the cause*. ⚠ **Do not reuse it** — it is `ConnectOutcome`-typed. Copy the *message shape*, not the helper. |
| `every-waiter-can-bound-its-wait/SCORE.md` | why `RecvOutcome::Stopped` is a reserved bucket (325 arms) and not yours to re-dispose at one site. |
| `wat-scripts/queue/sqs.wat:195` | this fold is inside a `defservice` `:impls` body ⇒ it runs in a **forked child**. No parent helper is visible (trap-door 3). |

## Implementation sketch

Nine named messages, each keeping `"queue: probe scan"` and adding what actually happened:

```
RecvOutcome::Lost c       → "… the store peer is GONE: {cause}"
RecvOutcome::Closed       → "… the store connection closed cleanly (no cause available)"
RecvOutcome::TimedOut     → "… the store is ALIVE AND SLOW — the deadline fired, not a death"
RecvOutcome::Stopped      → "… the WORLD IS STOPPING — this is a shutdown, not a failure"   ⚠ see below
RecvOutcome::Malformed c  → "… the store could not DECODE our frame (it is alive): {cause}"

ScanResponse::Transient e        → "… a RETRYABLE store error (transient-means-try-again applies): {e}"
ScanResponse::Fatal e            → "… a FATAL store error: {e}"
ScanResponse::RequestTooLarge b cap → "… OUR request exceeded the cap: {b} > {cap} — our defect"
ScanResponse::RequestMalformed p e g → "… OUR request did not decode at {p}: expected {e}, got {g}"
```

⚠ **`Stopped` keeps raising in this stone** — its *message* becomes honest, its *disposition* is reported
for a ruling. Do not quietly make it a no-op; 324 siblings stand.

## Blast radius

`wat-scripts/queue/sqs.wat` only, one fold. **Expected 0 elsewhere** — confirm rather than inherit.

## STOP triggers

1. ⛔ **STOP-1 — the `_` must GO, replaced by four named arms.** A prettier message over a still-collapsed
   wildcard is the defect restated. The checker forces exhaustiveness, so it will tell you if you missed one.
2. ⛔ **STOP-2 — do NOT change any disposition**, including `Stopped`'s. Report it.
3. ⛔ **STOP-3 — do NOT make `Transient` retry here.** That is `transient-means-try-again`'s territory and
   this is a probe-scan fold, not the store call path. ⭑ But **report whether it should** — a retryable
   error killing the queue in a fold is a finding this stone surfaces and does not own.
4. **STOP-4 — do NOT invent a cause for a nullary variant.** `Closed`/`TimedOut`/`Stopped` carry none;
   say only what the site knows.
5. **STOP-5 — no parent helper.** Forked child; self-contained or stdlib only.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**; delta as a **band**. ⛔ `timeout -k` on anything that
  might block.
- ⭐ **Nine distinct strings exist:** `grep -c 'peer is dead, not a broken pipe' wat-scripts/queue/sqs.wat`
  → **0**, and nine different new messages present.
- ⭐ **Driven where drivable:** the store-fault injector (`store-drop-reply-bp` / `store-die-bp`,
  `wat-scripts/query/faulting-store.wat`) can reach the `RecvOutcome` arms. **Drive at least one and quote
  the message.** For the `ScanResponse` variants, say plainly which are reachable and which are not —
  an unreachable arm is an honest report, a fabricated fixture is not.
- ⭐ **Happy path unchanged:** the record invocation → `distinct=8000;dup=0`, completeness counters identical.
- ⚠ Report whether `Transient` should retry here (STOP-3).

## Shape to copy

`a-dial-failure-says-which-of-three/SCORE.md` — the message shape (site + variant meaning + carried
cause) one enum over, struck an hour ago.
