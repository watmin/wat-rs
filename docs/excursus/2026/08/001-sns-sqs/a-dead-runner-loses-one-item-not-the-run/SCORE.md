# SCORE — a dead runner loses one item, not the run

**SCORED as a STOP.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `cb8bcfe46` (second DESIGN correction). Did not commit. Re-dispatch was not landed — it was started, then withdrawn when row 1b made it illegal.

Sentence: **One runner dying should cost one item, not the whole bracket.**

The sentence is still true. It cannot be struck until `select` stops reporting a garbled frame as a dead peer.

---

## Row 1 — seven arms, protocol vs transport

`collect-loop` calls `(:wat::kernel::select peers)` = `eval_peer_select_values`. The argument is a vector of **runner peers**: spawned workers whose wire type is `(PoolMsg, (Tuple i64 O))`. Not a listener. Not an owner-lineage socket.

Pinned: `kernel_select_builds_only_four_serviceevent_variants` — `select` builds `{Closed, Lost, Message, Shutdown}`. `poll` (bracket calls it **0** times) adds `{Admin, Connection, Malformed, Rejected}`.

| arm | class | evidence | this stone |
|---|---|---|---|
| `Connection` | **protocol-impossible** | A runner peer is a spawned worker's data channel, not a `Listener`. Connection is poll's accept arm. Honest at any distance, including `RemoteOpts`. | keep `assertion-failed!`; comment says PROTOCOL |
| `Admin` | **protocol-impossible** | A runner peer is not the owner-lineage socket. Admin is poll's self-peer arm. Honest at any distance. | keep `assertion-failed!`; comment says PROTOCOL |
| `Closed` | **transport fact** | `select` builds it (4 emitters). A network produces clean hangup as routine weather. | would be RETRY; **not landed** — see 1b |
| `Lost` | **transport fact** | `select` builds it. Crash, death-notice, **and decode_trusted_wire failure**. | RETRY **illegal** until the collapse splits — 1b |
| `Shutdown` | **transport fact** | `SelectOutcome::Shutdown` / `PeerDeath::Shutdown`. World stopping. | REPORT-GONE; surface has no slot; not landed |
| `Malformed` | **transport fact, but not this verb** | Built by `poll` only. `select` has no Malformed — it folds garbled into Lost. Shared `ServiceEvent` is a too-wide enum (filed, not fixed). | panic is the only honest match on a variant the verb cannot construct |
| `Rejected` | **transport fact, but not this verb** | `poll` FrameTooLarge only. Same too-wide enum. | same |

⛔ **"Local IPC cannot produce this" was not used as a reason to assert.** Connection/Admin assert from what a runner peer *is*. Malformed/Rejected panic because *this verb cannot construct them*, which is a type-shape fact (one enum, two verbs), not a locality fact.

---

## Row 1b — ⭐ THE STOP: `Lost` already means two things

`src/runtime.rs` on the process-tier `select` decode path (comment at the `decode_trusted_wire` `Err` arm):

> *"A peer whose frame will not decode is dead (recv:25047), not a live client with a junk message (poll:27194 Malformed)."*

That is not a hypothesis. `select` **does** put a decode failure on `Lost`. Over IPC a trusted wire is near enough. Over a network it is false: a healthy remote runner that emits one corrupt frame is reported dead.

**Cannot establish that bracket's `Lost` cannot carry a decode failure** — the constructor is in the `select` path collect-loop actually calls.

A naive RETRY-on-`Lost` would therefore re-dispatch on:

1. a runner that actually died (correct), **and**
2. a deterministic encode bug collapsed into `Lost` (forbidden — `Malformed` is never retried, and the handler never sees `Malformed`)

until the wall clock expired. That is the defect `a-momentary-failure-is-not-fatal` exists to undo, one tier lower, **before** this handler can be correct.

**Prerequisite, named:** split `select`'s decode-failure-to-`Lost` collapse (give the pool select an untrusted-frame outcome, or stop calling `decode_trusted_wire` a trusted-peer fact). Until that lands, this stone does **not** re-dispatch.

Re-dispatch was started on HEAD `058d8bf99` / `cb8bcfe46` and **withdrawn**. `collect-loop` is restored to raise on Closed/Lost with the holding-phrase. The mutation tests that expected `[0 2 4 6]` after a worker panic are deleted — they would have certified the illegal RETRY.

---

## Rows 2–5 — not struck

| row | why not |
|---|---|
| 2 taxonomy | Closed/Lost RETRY cannot be graded on a `Lost` that means two things. Malformed is not this verb's; poll's Malformed still cannot distinguish sender-garbage vs wire-damage (finding against the governing DESIGN, inherited, not re-decided). |
| 3 re-dispatch | blocked on 1b |
| 4 mutation control | would certify the illegal RETRY; not built |
| 5 non-vacuity | n/a |

Closed-only RETRY was considered: the control that kills a runner mid-flight (work-fn panic) is **Lost**, not Closed. Closed-only would not deliver the sentence.

---

## Row 6 — surface untouched

`map` / `each` signatures unchanged. No widening.

---

## Row 7 — scope wall

Did **not**:
- touch the ten dead `RecvOutcome` arms (`probe_bare_recv_outcome_surface.rs` still pins them)
- strike stone 1b (`Failed` still on `<S>::Reply`)
- change `map` / `each` signatures
- touch the queue path
- split the `select` collapse (that's the named prerequisite, a different stone)
- invent a narrower pool-select event type (filed in the DESIGN correction, not this strike)

---

## Row 8 — floor

After restoring `collect-loop` and deleting the illegal-RETRY tests. Summary + `.floor/` filled in after `scripts/floor.sh`.
