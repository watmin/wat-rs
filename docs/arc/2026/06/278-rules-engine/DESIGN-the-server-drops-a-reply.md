# DESIGN — the server drops a reply

**Stone R2.** The last fault. A rate-gated drop inside the seam, which is the only placement that
produces *work-done, caller-unaware*.

## WHY — everything is finally in place

| piece | stone | state |
|---|---|---|
| the seam at the reply-send | R1 v3 | ✅ `send-keep-serving?` at `service.wat:3108`, five callers |
| the client survives a silent server | T1 | ✅ times out, discards, redials, retries on a **fresh** peer |
| the ledger can witness absorption | S30 | ✅ `seen-firsts` / `seen-dups` |
| seeded replay from wat | 3a/3c | ✅ `SEEDED-CHAOS-IS-REPLAYABLE` |
| `Closed` recovers instead of dying | 3c-pre | ✅ 17 arms |

The tracker's table has had one empty row since the arc opened:

| drop lands | work happened? | caller knows? | duplicate on retry? |
|---|---|---|---|
| before dispatch | no | no | no ← **3c, measured, `seen-dups=0`** |
| **after the arm, before the reply-send** | **yes** | **no** | **YES** ← this stone |

`seen-dups` has never moved outside a deterministic gate. **T1 gave the client a retry; R2 gives it
something to retry.**

## ⛔ THE ONE CONTRACT DECISION — AND I DO NOT KNOW THE ANSWER

**Where do the rate and seed live?**

`send-keep-serving?` is a plain `defn` with no state. A rate-gated drop needs a rate and a seed, and
**the seed must advance and replay** — 3c's rule: *chaos you cannot replay is chaos you cannot
debug.* Three candidates:

| option | cost |
|---|---|
| **(a) `:durable` fields on the service Record** | every `defservice` in the corpus grows two fields, unless the template can pass a no-drop constant for services that do not declare them |
| **(b) an ambient config the helper reads** | needs a config-read mechanism **I have not verified exists** |
| **(c) ambient `:wat::rand::int`** | ⛔ **REJECTED** — Pure but **not Deterministic**. No replay, no debugging |

★ **The first act is a probe that settles this**, not a design meeting. **Is the service's `state`
in scope at the five call sites?** If yes, (a) is reachable and the seed threads through the loop's
existing state. If no, (a) is dead and (b) must be established or the stone re-shaped.

I looked and could not establish it from the template's structure. **I am shipping the absence
rather than a guess** — that is what v1 of R1 failed to do, and it cost a strike.

## THE SHAPE — once the decision lands

```wat
;; inside send-keep-serving?, before the send:
;;   draw from the seed; if draw < rate-bp, DO NOT SEND, and still return TRUE
;;   -- a drop is not `Stopped`; the loop keeps serving, the caller learns nothing
```

**Returning `true` is the whole fault.** `false` stops the world; `true` means *"I handled it"* while
the caller is left waiting — which T1 turns into a timeout, a discard, a redial, and a retry.

## WHAT IT SHOULD PRODUCE

1. The server drops a `Seen/claim` reply after writing the ledger.
2. The worker's deadline fires; it discards, redials, **retries the claim**.
3. The retry is a **second claim on the same seq** → `Dup` → **`seen-dups > 0`.**

★ And the invariant should hold: `first? = false` on the retry means no second outcome is emitted, so
`distinct` stays 8000 and `dup` stays 0. **`seen-dups` moves; nothing else does.** If `distinct`
drops, that is loss and it is the finding.

## OUT OF SCOPE = REJECTED

- **Server-side handle killing / evicting from `selectables`.** The helper returns *keep-serving?* — a
  `bool` about the loop, not about that peer. Expressing eviction needs a richer outcome. **S39.**
- **Adopting the drop on any call but `Seen/claim`.** T1 only gave `claim` a deadline; a drop on a
  call with no deadline is a hang. **The two must stay matched.**
- **Rate anything but 0 by default.** A floor that silently drops replies is 3c-pre's ARM again.

## THE PROOF

1. **★★ `seen-dups > 0`.** The number that has never moved. Any non-zero is the result.
2. **★★ The placement discriminates.** A drop *before* the ledger write must leave `seen-dups = 0` —
   the claim never landed, so the retry is a `First`. Same rate, same seed, one variable. **If both
   cells agree, the placement was never the variable**, which is the whole table.
3. **★ `distinct = 8000`.** ⛔ Below it is **loss**, and the only genuine failure on this row.
4. **Rate 0 unchanged** — floor `5214/5214`, `seen-dups=0`, no drop armed.
5. **The seed replays** — two runs, same seed, same `seen-dups`.
