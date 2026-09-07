# DESIGN — the ack retries like the publisher does

**The ack path gets the retry discipline the publish path already has, bounded by the redelivery
window it is racing.** `wat-scripts/fanout/circuit.wat` only.

## WHY — the same problem, solved once and not the other time

`fill deep, then drain` failed at n=2000 with `[0/0][0/0][0/0][0/20]` — 30 messages (grok's run) and
20 (mine) claimed and never acked, both **multiples of 10** because the ack is batched per
`receive :limit 10`. The drain waits on `unacked = 0`; it never comes.

The ladder, `:604-607`:

```wat
aa1 (once-a q)
aa2 (if (second aa1) (once-a (first aa1)) aa1)
aa3 (if (second aa2) (once-a (first aa2)) aa2)]
   (Tuple (Tuple (first aa3) seen2 outs1) 0)
```

`second aa3` — the retry flag — is **discarded**. Three tries at a flat 200 ms deadline (`:595`),
then silence.

★★★ Now read the **publish** path in the same file (`:1126-1152`), which solves the identical
problem — a client bouncing off a busy server:

| | publish | ack |
|---|---|---|
| backoff | `backoff-delay` — exponential, jittered, capped (`:1109`) | none; flat 200 ms |
| attempts | until a **time** bound | exactly 3 |
| exhaustion | `assertion-failed!` carrying a verdict | **silently discarded** |
| instrumented | `retries`, `asleep`, `attempts` | nothing |

⚠ `backoff-delay` has **exactly one caller** — `:1142`, the publisher. The builder asked for it by
name: *"i don't want a magic value … intelligent values must have increasing backoff … jitter …
cap."* The publish loop got it. The ack loop, the other retry loop in the same file, kept three
flat constants.

## ⛔ THE INSTRUMENT I ALMOST BUILT, AND WHY IT WOULD NOT READ

The obvious stone is "count abandoned acks." **It cannot be read.** An abandoned ack leaves the row
unacked; the drain requires `unacked = 0`; so **any run that completes had zero abandonments**, and
any run that abandons panics at `require!` *before* the summary prints. The counter would read `0`
on every run it could be read on.

★ The readable instrument is **`ack-retries`** — attempts beyond the first. Those happen on passing
runs, scale with contention, and are the leading indicator of the terminal event.

## ⛔ THE ONE CONTRACT DECISION — the bound is the redelivery window

The ack retries with `backoff-delay` until it succeeds **or until `vis-ns` has elapsed** — the point
at which the message becomes visible again and someone else will take it. After that instant,
retrying is not merely bounded, it is **pointless**.

```
limit-ms = vis-ns / 1_000_000
```

★★★ This is derived from the system's own semantics, and it makes both regimes correct **with no
flag and no branch**:

- **chaos runs** (`drop-ack-bp > 0`) set `vis = 200 ms` (`:1868-1875`). The ack gets ~200 ms of
  backed-off retries and then stops — and vis expiry redelivers, which is exactly the recovery
  those tests exist to exercise. Behaviour preserved **by construction**.
- **normal runs** set `vis = 10¹² ns`. The bound is effectively "retry until acked", which is what
  we want, and a genuinely dead queue is caught by the drain's own liveness bound instead.

⚠ So exhaustion needs no special case: where it is legitimate, redelivery follows; where it is not,
it cannot be reached before a higher bound fires. **The wrong behaviour has no form to take.**

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

★ **The gate is binary and already failing:** `2000 4 3 8192 true` completes, or it does not. That
run reproduces on demand, in both our hands, and it is the exact run this stone exists to fix.

⚠ **This is not predicted to move the curve.** At n ≤ 1000 no ack is abandoned today, so backed-off
retries should cost nothing there. If `pairs/sec` at n=100–1000 moves materially, that is a finding
to explain, not a win to bank.

⚠ **It does not explain the 25 % slope** from n=100 to n=1000. That slope was measured with zero
abandonments and is a separate open question.

## OUT OF SCOPE — REJECTED

- **Shortening `vis` on normal runs.** That would give abandoned acks a redelivery net, but it also
  changes what the circuit is testing (the header sets `vis` long precisely so the happy path never
  redelivers). A different ruling.
- **The 200 ms deadline on the individual call.** It is a reasonable liveness probe for one attempt;
  the defect is the *ladder* above it, not the rung.
- **Promoting `backoff-delay` to `wat/`.** The tracker already names the eight hand-rolled userland
  helpers as belonging in the stdlib. Still true, still its own stone.
- **The receive path's 1000 ms deadline** (`:452`). Untouched; a receive timeout leaves the message
  visible, which is already safe.
- **Counting abandonments.** Unreadable by construction — see above.
