# SCORE — the probe scan says which of six (nine, measured)

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `0ebdfa334` (AMEND: caller-caused refusal is first-class). Did not commit.

⛔ The headline is that **the diagnosis is named. These arms still raise.** `Stopped` is reported, not re-disposed.

```
     Summary [ 560.656s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T21-14-40Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Floor delta is a band, not a number (EXPECTATIONS).

Production blast: `wat-scripts/queue/sqs.wat` only (one fold). Drive probe untracked under scratch-pad.

---

## ⭑⭑ THE HEADLINE — nine worlds, nine messages, `_` gone

`grep -c 'peer is dead, not a broken pipe' wat-scripts/queue/sqs.wat` → **0**.

Site label `"queue: probe scan"` survives. Causes carried where the variant has one. Nullary arms invent none.

```
ScanResponse::Transient e        queue: probe scan: a RETRYABLE store error (transient-means-try-again applies): {e}
ScanResponse::Fatal e            queue: probe scan: a FATAL store error: {e}
ScanResponse::RequestTooLarge    queue: probe scan: this caller oversized the request: {b} > {cap}
ScanResponse::RequestMalformed   queue: probe scan: this caller sent a frame that did not decode at {p}: expected {e}, got {g}
RecvOutcome::Lost c              queue: probe scan: the store peer is GONE: {cause}
RecvOutcome::Closed              queue: probe scan: the store connection closed cleanly (no cause available)
RecvOutcome::TimedOut            queue: probe scan: the store is ALIVE AND SLOW — the deadline fired, not a death
RecvOutcome::Stopped             queue: probe scan: the WORLD IS STOPPING — this is a shutdown, not a failure
RecvOutcome::Malformed c         queue: probe scan: the store could not DECODE our frame (it is alive): {cause}
```

Hand edit, not a codemod. No helper (forked child; six arms in one place). Copied `redial-failed!`'s *message shape*, not the ConnectOutcome helper.

---

## STOP-1 — the `_` is gone

Four named `ScanResponse` arms. Checker exhaustiveness is the gate; `--check` on `sqs.wat` is 0.

---

## STOP-2 — no disposition changed

All nine still `assertion-failed!`. `Stopped` included. Diff is messages + the four new arms.

---

## STOP-3 — `Transient` is not called a death; should it retry here?

The message names it **retryable** and cites `transient-means-try-again`. Not retried (this is the probe-scan fold, not the store put/delete path).

⚠ **View, not a change:** it **should** retry, or at least not kill the queue. `transient-means-try-again` retried put/delete in this same file specifically so an honest SQLITE_BUSY does not die. This fold is a post-put completeness count; a Transient here is the same class of error, and dying on it undoes that stone at one site. Builder rules it. Not this stone.

---

## 4b — caller-caused refusal: what the disposition *should* be (not changed)

Arc 278 Stone 2 made the **receiving** half crash-proof: a bad caller gets `RequestTooLarge` / `RequestMalformed` as a **value**, with `bytes`/`cap` or `path`/`expected`/`got`. This site is the **sending** half of that doctrine, and it fails it: the store refused honestly, and the queue **throws that away, blames nothing useful, and dies** — still, after this stone, via `assertion-failed!`.

What a caller should do, holding that value:

- **Not raise.** Dying while holding the numbers needed to fix the request is the sending-half failure of a receiving-half ruling already struck.
- **Surface it to its own caller as the same class of value.** `:queue::Queue::SendResponse` already has `:RequestTooLarge` and `:RequestMalformed`. The diagnosis has a home one layer up.
- This fold is a post-put completeness count, not the original send. A refusal here means "the probe scan of a row we just wrote was rejected as our frame" — still a caller-caused refusal, still not a death. Count it not-landed, or return the typed send failure. Do not kill the queue.

⛔ Disposition **unchanged** this stone (STOP-2). Finding, not a fix. Same shape as `Transient`-should-retry and `Stopped`-needs-a-ruling.

---

## STOP-4 — no invented causes

`Closed` / `TimedOut` / `Stopped` are static strings. `Lost` carries `LociDiedError/message`. `Malformed` carries `Failure/message`. `Transient`/`Fatal` carry `edn::write` of the record. `RequestTooLarge`/`RequestMalformed` carry the numbers/path the variant holds.

---

## Driven where drivable

`store-drop-reply-bp` / `store-die-bp` **cannot reach this fold.** `faulting-store.wat` injects on put/delete only; scan always pass-through. Those knobs fire before `count-landed`. Honest, not fabricated.

Drove the fold with a scan-fault wrapper (put still succeeds, scan faults):

```
TRANSIENT=Lost:service severed: its owner released the service handle
DIE=Lost:service severed: its owner released the service handle
```

Both paths entered `count-landed` (otherwise send would `Accepted`). The arm still raises; D1-a's handler wrap turns that raise into a graceful stop, so the **client** sees `Severed`, not the assertion text. The named strings are the assertion payloads (quoted above). Not a disposition change in this fold.

ScanResponse reachability:

| variant | reachable? |
|---|---|
| Transient | **yes** — wrapper returned it; fold raised |
| Fatal | same shape, not separately driven |
| RequestTooLarge / RequestMalformed | this caller's frame on a 1-row point-scan; not induced, not fabricated |
| RecvOutcome Lost/Closed | DIE=store `Stop` on scan; queue saw a gone peer and raised |
| TimedOut | would need a silent live store on **scan**; injector does not; ~10 s per; not fabricated |
| Stopped | reserved class; not induced |

---

## Happy path

`n=2000 m=4 j=3 fill-first`:

```
total=8000;distinct=8000;dup=0
seen-recorded=8000;seen-skipped=0
```

---

## `Stopped` for a ruling

Raising on a shutdown is wrong under the builder's doctrine (nothing failed). This site now **says so**. Disposition unchanged: 324 siblings stand, and `every-waiter-can-bound-its-wait` reserved the bucket.

---

## WHAT LANDED

- `wat-scripts/queue/sqs.wat` — one fold, nine named raises
- scratch-pad drive probe (untracked)
