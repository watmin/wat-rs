# DESIGN — a reconnect is not an abandonment

**`Lost` and `Closed` redial and RETRY, instead of redialing and discarding the work.** All three
retry ladders. `wat-scripts/fanout/circuit.wat` only.

A **correctness fix**, not an instrument.

## ⛔ WHY — the code reconnects and then throws the work away

Every ladder in the worker has this shape (ack at `:709-712`, `seen-until` at `:532-534`):

```wat
((:wat::service::CallOutcome::Answered _r)   (… Got …))
((:wat::service::CallOutcome::Lost _c)       (… Exhausted 0, (redial-q) …))   ;; ← abandons
((:wat::service::CallOutcome::Closed)        (… Exhausted 0, (redial-q) …))   ;; ← abandons
((:wat::service::CallOutcome::DeadlineFired) (… retry until the time bound …))
```

★★★ **`Lost` and `Closed` call `redial` — obtaining a fresh, working peer — and then return
`Exhausted`, discarding the message.** It reconnects and throws the work away.

⚠ And this project already ruled on what `Lost` means. There is a commit named
**"a peer is dead only when redial fails"**, and `redial` itself asserts on genuine death:

```
"queue: redial failed — peer is dead, not a broken pipe"
```

So the two cases are already separated by the substrate: **redial succeeds ⟹ the peer is alive**;
redial fails ⟹ an assertion fires. There is no third case in which discarding is correct.

## ⛔ THE EVIDENCE — this is what strands messages

`the dedupe map stops cloning itself` did not drain at n=2000: `[0/0][0/10][0/20][0/10]`, **40
unacked = 4 batches of 10**, with `ack-retries=6`.

★★★★ **Those cannot be deadline exhaustion.** `ack-limit-ms = vis-ns / 1e6 = 1 000 000 ms`, so the
`DeadlineFired` path retries for up to **1000 seconds** and essentially cannot exhaust. The only
path that abandons quickly is `Lost`/`Closed` — **zero retries**. `ack-retries=6` is the healthy
deadline path working; the 40 stranded are a different path entirely.

⚠ **I flagged this and deferred it.** The grading of `the outcome crosses, the resource stays` says:
*"a broken pipe would be reported as an abandonment — the same naming lie as `gave-back`… not
observed firing."* It is now observed firing, and it is what strands messages.

## ⛔ THE ONE CONTRACT DECISION — `Exhausted` means "out of time", nothing else

`Lost` and `Closed` redial and **retry**, on the same time bound and the same backoff the
`DeadlineFired` arm already uses. `Exhausted` becomes reachable **only** from the bound.

★ That is the invariant, stated positively: **a reconnect is not an abandonment.** A ladder gives up
for exactly one reason — it ran out of time — and every other outcome either succeeds or retries.

### And the counter that would have caught it

`worker::Record` carries `check-exhausted`, `mark-exhausted`, **`ack-retries`** — there is **no
`ack-exhausted`**. So the ack path counts its *retries* and not its *abandonments*, which is why the
failure line reports `ack-retries=6` while 40 messages sit stranded.

⚠ **That gap is mine.** The ack stone's DESIGN argued a counted abandonment "cannot be read" because
a completing run implies zero — true for that stone, and wrong once grok put counters on the failure
path, where it is exactly the number needed. `ack-exhausted` joins its two siblings.

## WHAT THIS IS AND IS NOT

⚠ **It is not a timeout change.** `vis` at 1000 s on a ~40 s run is separately illogical — it is 25×
the run, and it makes any abandonment unrecoverable inside the test rather than slow. **That is a
different stone**, and fixing it first would *mask* this defect by letting stranded messages
reappear after 1000 s.

⚠ **It does not explain the slope.** `the dedupe map stops cloning itself` is unmeasured at n=2000
because of this bug; the slope question resumes once n=2000 completes.

## OUT OF SCOPE — REJECTED

- **`vis` on non-drop runs.** Illogical at this scale, real, and its own stone — *after* this one,
  so the fix is judged on the bug rather than on a masking constant.
- **The `Lost`/`Closed` distinction.** Both mean "the pipe broke and I reconnected"; neither needs
  its own arm here. If they ever need to differ, that is a separate ruling.
- **`circuit.wat:2075`** (the `distinct` fold) and **`wat/query/mem.wat`** — the same O(N²) map
  clone. Named, untouched, `wat/` is the builder's call.
