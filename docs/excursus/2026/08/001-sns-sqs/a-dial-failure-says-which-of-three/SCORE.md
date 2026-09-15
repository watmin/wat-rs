# SCORE — a dial failure says which of three things happened

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `8d5787806` (DRAWN). Did not commit.

⛔ The headline is that **the diagnosis is named. A dial failure still raises.**

```
     Summary [ 562.268s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T19-01-04Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Floor delta is a band, not a number (EXPECTATIONS).

---

## ⭑⭑ THE HEADLINE — Refused / Rejected / Failed, site + cause

Helper (`require-stopped`'s shape: whole outcome in, peer out on Connected):

```
queue: redial: dial REFUSED (nothing listening): {cause}
queue: redial: dial REJECTED — a DIFFERENT process holds that address (stale capability, NOT a death): {cause}
queue: redial: dial FAILED (io error reading peer-cred): {cause}
```

Site label survives. Cause survives. Rejected says stale address, not death.

Picked this shape over `(_ (redial-failed! site o))` because `_` does not bind `o`, every collapsing Connected arm was identity on the peer, and `require-stopped` already is "outcome in, payload out, raise named."

---

## Census is the authority: **35**

`wat --grep` on the three service scripts: **35** `collapsing-connect-catchall` (sqs 7, sns-fanout 16, circuit 12). Agreed with the brief's 35 this time; the extra 6 `"peer is dead"` tokens in `sqs.wat:594–614` are **RecvOutcome / ScanResponse**, not Connect — out of scope, not rewritten.

Second apply: `n=0` on all three. Grep after: **0**. Idempotent.

---

## STOP-2 — `_` not rewritten

`circuit.wat:3848` `(_ "redial-failed")` — returns a label, does not raise. Left.

`tests/services/probe_dial_declares_its_peer_redial.wat` and three scratch-pad probes still collapse with the old string. Outside the three-file apply list. Named, not swept.

---

## STOP-4 — the 18 named sites

Still `ConnectOutcome::Refused` × **18** in the three files (e.g. `sqs.wat:251` init dial, `:1929` `:user::dial-queue`). Untouched. They already name the three variants and carry `Failure/message`. They do **not** carry a site label. A later stone could point them at the helper for the label; not this one.

---

## Driven / unreachable

- `probe-crash-surface-connect-refused.wat`: `connect-after-stop=Refused` (unchanged).
- Helper: `probe: redial: dial REFUSED (nothing listening): connect abstract UDS: Connection refused (os error 111)`. Still raises (exit=2).
- **Rejected**: crash-surface matrix UNREACHABLE from userland. Not fabricated. Honest pass.

---

## Happy path

`n=8000 m=4 j=3 fill-first`: `distinct=32000;dup=0` (8000 per sub × 4). `pub-exh-last=none`.

---

## Comment pass

Codemod walks forms; comments untouched. Manual: left `sns-fanout.wat:653` / `circuit.wat:1173` ("a peer is dead only" — Lost vs death, not the collapsing lie). Left the six RecvOutcome probe-scan strings.

---

## WHAT LANDED

- `wat/service.wat` — `:wat::service::redial-failed!`
- `wat-scripts/fixes/redial-failed-names-the-variant.wat` — recorded migration
- 35 sites in `sqs.wat` / `sns-fanout.wat` / `circuit.wat`
- scratch probe driving Refused through the helper

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | three variants named | helper match Refused/Rejected/Failed |
| 2 | cause survives | `{cause}` = `Failure/message c` |
| 3 | Rejected = stale address | "DIFFERENT process holds that address (stale capability, NOT a death)" |
| 4 | site label survives | `"queue: redial"`, `"topic-worker: redial inbox"`, … |
| 5 | still RAISES | helper Refused exit=2 |
| 6 | recorded idempotent codemod | second apply n=0 |
| 7 | census authority | **35**, with matches; 6 RecvOutcome extras named |
| 8 | 18 untouched | Refused still ×18 |
| 9 | Refused driven | site + REFUSED + os error 111 |
| 10 | Rejected reachability | UNREACHABLE, not fabricated |
| 11 | happy path | dup=0, n=8000 |
| 12 | floor 5251 | Summary **5251 passed** |
| 13 | clippy 0 | CLIPPY=0 |
| 14 | comment pass | stated above |
