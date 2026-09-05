# DESIGN — the death notice is not a malformed frame

**Both remaining floor reds, one chain.** `src/runtime.rs`. Correctness. No perf work.

## THE TWO REDS

```
rst      wants RecvOutcome::Lost (peer crashed)      gets Ok(String("CLOSED"))
severed  wants LociDiedError::Severed                gets "CLOSED:MUTE"
```

## ⛔ WHAT THE UNDECODABLE FRAME ACTUALLY WAS

`kernel/peer.rs:460-485` — a dying peer sends a **reserved keyword Value**,
`PEER_CRASHED_SENTINEL` / `PEER_SEVERED_SENTINEL`. The original failure was:

```
EDN parse error: invalid keyword: keyword begins with ::
```

★ **That was the death notice.** `select` did not fail to receive it — it received it and could
not read it. The bytes were never the problem.

## THREE CHANGES, EACH WITH AN EXEMPLAR

### 1. Process tier: an undecodable peer frame is `Lost`, not `Malformed`

`recv:25047` turns a decode failure into `RecvOutcome::Lost`, and its comment says why: *"the
crash reason is the full `#wat.kernel/ProcessPanics [...]` envelope text."* **`recv` does not
recognize the sentinel at process tier either — it treats undecodable-from-a-peer as death, and
that is correct.**

⚠ **My previous DESIGN sent the executor to the wrong exemplar.** I cited `poll:27194`
(`Malformed`) for this arm. `poll`'s `Malformed` is for a **live client's** bad message — the
connection is fine, the message is junk. A **peer** whose frame will not decode is **dead**.
Grok followed my citation exactly; the mis-citation is mine.

### 2. Thread tier: intercept the sentinel, as `Peer::recv` does

`peer.rs:432` `is_peer_crashed_sentinel` / `:443` `is_peer_severed_sentinel` — its own comment
calls itself *"the ONE place to recognize it"*, because thread-tier peers carry `Value`
in-process. `select` reads those same `Value`s and never checks. **That check is what turns an
owner-drop into `Severed` rather than a generic death** — and it is the only path that can
produce the variant the severed test names.

### 3. `CallOutcome` stops merging — the cause must survive

`PeerGone` becomes `Lost [cause]` and `Closed`. Today the generated method flattens both to
`RecvOutcome::Closed`, which is why **both** reds report `CLOSED`.

★ **This is my debt.** I introduced the merge in the CallOutcome stone and graded it *"not a
defect today, but the shape permits one."* Four call sites, all currently treating both
identically, so they collapse at the call site instead of in the type.

## ⛔ THE ONE CONTRACT DECISION

**A peer's death is reported as a death at every tier, and the client still learns no cause it
should not have.**

Arc 294 (`runtime.rs:6558`) holds unchanged: `Severed` and `Stopped` pass through as
payload-free variants; everything else is reason-free. **The distinction is routing information
the client needs — `Severed` means redial fails identically. The payload is not.**

## FILES

`src/runtime.rs`.

## OUT OF SCOPE = REJECTED

- **Changing `ServiceEvent` or `LociDiedError`'s variant sets.** Every needed variant exists.
- **`poll:27124`** (admin channel) — different channel, different question.
- The rung-3 migration, checkpointed at `276f989dc`. All perf work.
