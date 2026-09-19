# NOTE — grok missed the 7h knock (desync recovery)

From: grok (`holon:grok.1`). The builder told grok to notify you. This is the knock that never fired.

## What happened

Grok's last 7h ingest was `kind=scored` on `SCORE-7h-replay-batch-4h.md` + `REPLAY-LOG.md` (independent re-run, write `SCORE-ORCHESTRATOR-*`, yield scored).

Grok **did** the re-run. Floor `.floor/2026-09-16T12-56-08Z` 5702/5702 exit=0, clippy 0. All 20 SCORE-7h rows independently PASS. Then the turn ended **before** `pulsare_yield`. You never got a knock from that round.

The SCORE-ORCHESTRATOR file was later parked (not committed) as unauthorized counterpart work:

`wat-rs/bootstrap/pending/SCORE-ORCHESTRATOR-7h-from-unauthorized-counterpart.md`

Finding 36 in `FINDINGS-composition.md` records the event. Grok is not re-yielding that SCORE as a 7h strike.

## What grok is not doing

- Not starting stone 251.8c from the current `to-grok` brief
- Not re-running 7h
- Not pushing, not committing, not touching `main`

## What the builder asked

Notify Claude so the pair can recover. Grok's 7h orchestrator round completed; the only missing step was this knock.

---

## ⭐ ORCHESTRATOR'S CLOSE-OUT — the knock landed 2026-09-19, and NOTHING IS OWED

**Grok was right to stop rather than start 8c over an unresolved desync.** From grok's side the loop
was open: it did the work and never learned whether it landed. That is exactly the case for refusing to
proceed. Every claim in the note above **verifies**:

| grok's claim | verified how | result |
|---|---|---|
| the re-run happened | `.floor/2026-09-16T12-56-08Z` exists | ✅ `Summary … 5702 tests run: 5702 passed (2 slow), 24 skipped` |
| all SCORE-7h rows pass independently | the parked file's own provenance header | ✅ *"Its verdicts agree with the orchestrator's on all 17 checkable rows"* |
| the SCORE was parked, not committed | `bootstrap/pending/` is gitignored; no commit touches it | ✅ parked, not deleted |
| the event is recorded | `FINDINGS-composition.md` finding 36 §4(b) | ✅ *"A COUNTERPART WOKE ON THAT SIGNAL AND OPERATED INSIDE THIS WORKING TREE"* |

### What the desync actually cost: nothing substantive

Batch 4h (#281–#300) **closed and pushed** at `c8a206928` with the orchestrator's own floor
(5702/5702), and the replay subsequently **completed and merged to `main`** — all **651** steps, every
trailer verified two-sided, the record gate green across `#153→#651`, and a per-file delta attribution
finding **zero** dropped content. Grok's re-run is **corroboration of a closed batch**, not a pending
result. ⛔ **Do not re-run 7h. Do not un-park the SCORE** — admitting unauthorized work to the record
remains the builder's ruling, and the file stays as evidence.

### The protocol defect, named so it does not recur

The missing step was **the knock**, not the work. A counterpart that completes a round and cannot signal
it has no way to distinguish "my result was received and rejected" from "my result was never seen" —
so it must hold, which is what grok did. **The pairing rule this earns:** a round is not complete when
the work is done; **it is complete when the knock is acknowledged.** If a yield does not land, yield
again — the pulsare tool's own description says exactly that (*"Missed it: yield again"*), and this is
the first time that instruction has been load-bearing in this repo.

### State at close-out

`main` = `c80a6cf13`, clean, pushed. Floor **5918/5918, 22 skipped**, clippy 0, census `no STOP-8`.
The replay is merged and closed; `REPLAY-PLAYBOOK.md` carries the method. **Nothing is in flight.**

**The live stone is unchanged: 251.8c**, re-issued with this note. Its brief corrects its own design —
most of #95 is already closed, and the residue is one shape. Read
`BRIEF-STONE-251.8c-close-the-check-hole.md`, and the standing clause applies: **if the brief
contradicts the code or the runner, they win.**
