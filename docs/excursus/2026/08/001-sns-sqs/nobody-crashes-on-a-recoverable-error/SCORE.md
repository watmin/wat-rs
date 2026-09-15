# SCORE — nobody crashes on a recoverable error (the census)

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `745a4ff24` (DRAWN). Did not commit.

⛔ The headline is that **a census was taken. Nothing was made tolerant.**

```
     Summary [ 556.161s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T06-24-30Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Porcelain: `wat-scripts/census-recoverable-raising.wat` + this stone's FINDING / SCORE / run logs.
`src/` `wat/` `wat-scripts/{queue,topic,fanout}` **EMPTY**. Nothing migrated.

Executor wall 556 s is not comparable to the orchestrator's 538.417 s baseline (EXPECTATIONS).

---

## ⭑ THE HEADLINE — 2276 recoverable raising arms, classified; 0 arms migrated

```
recoverable keywords=3323; match-arms=3304; raising-arms=2276; unclaimed-arms=1028
STOPPED-BUCKET keywords=493; match-arms=492; raising-arms=325; unclaimed-arms=167
client-faces-service=2273  service-faces-client=3
```

All 2276 raises are `assertion-failed!`. Direction is derived from the family.

| family | raising | live (wat + wat-scripts/service) |
|---|---|---|
| RecvOutcome | 1471 | 123 |
| ConnectOutcome | 795 | 84 |
| CallOutcome | 5 | 5 |
| ServiceEvent | 3 | 3 |
| SendOutcome | 2 | 0 |
| TrySendOutcome | 0 | 0 |

tests+scratch-pad+docs+other = **2061 / 2276**. The D5 544-vs-59 shape repeats.

`defservice × wat-scripts/service` = **51** (all recoverable). D5's Malformed-only cell, re-run today: **17**.

---

## Controls, named per family

| family | PRESENT | ABSENT |
|---|---|---|
| RecvOutcome | `stdio.wat:221` Lost **PRESENT** | `circuit.wat:544` Malformed **ABSENT** |
| SendOutcome | `probe_a_peer_remembers_its_address.wat:44` Closed **PRESENT** | `circuit.wat:3873` Closed **ABSENT** |
| TrySendOutcome | **NONE-IN-CORPUS (STOP-1)** | `service.wat:2584` Closed **ABSENT** |
| CallOutcome | `circuit.wat:1015` Malformed **PRESENT** | `probe_a_fired_deadline…:65` Lost **ABSENT** |
| ConnectOutcome | `circuit.wat:414` Refused **PRESENT** | dying-client `:109` Refused **ABSENT** |
| ServiceEvent | `bracket.wat:626` Closed **PRESENT** | `circuit.wat:3857` Closed **ABSENT** |
| RecvOutcome::Stopped (own bucket) | `circuit.wat:543` **PRESENT** | `bracket.wat:53` **ABSENT** |

D5-inside-widened: circuit.wat:544 Malformed ABSENT, sns-fanout.wat:**421** Malformed ABSENT, wat/service.wat Malformed PRESENT.

**STOP-1** is TrySendOutcome PRESENT only. 15 match-arms, 0 raising, in the four homes and in `wat-tests/`. ABSENT fires. The 0 is a walker count plus a grep, not a PRESENT-controlled finding. Do not migrate it on this census.

STOP-5 PRESENT. Raising-arms is not complete for `let`/`do`/`if` bodies.

---

## D5 instrument, unchanged

```
CONTROL-A circuit.wat:544 raising-Malformed: ABSENT
CONTROL-A sns-fanout.wat:369 raising-Malformed: ABSENT
CONTROL-B wat/service.wat raising-Malformed: PRESENT
keywords=495; match-arms=492; raising-arms=466; unclaimed-arms=26
```

Controls pass. Cited D5 numbers (482/479/464/15, STOP-5 ABSENT) drifted with the corpus. Widened `RecvOutcome::Malformed` raising = **466**, matching this re-run. sns CONTROL A line **369 has drifted to 421** (369 ABSENT vacuously). CONTROL B cited line 3281 is now **3524**; D5's check is file-level.

---

## FLAG `service.wat:2549`

Appears unclaimed (`raises=UNCLASSIFIED`, body is `do`). Not in raising rows. Unreachable-by-unknown-kind. **Not ranked.**

---

## Exemplars (direction × family)

- client × RecvOutcome: `wat/kernel/services/stdio.wat:221` Lost
- client × SendOutcome: `tests/comms/probe_a_peer_remembers_its_address.wat:44` Closed
- client × TrySendOutcome: no raising row; unclaimed `wat/service.wat:2584` Closed nil
- client × CallOutcome: `wat-scripts/fanout/circuit.wat:1015` Malformed
- client × ConnectOutcome: `wat-scripts/fanout/circuit.wat:414` Refused
- service × ServiceEvent: `wat/bracket.wat:626` Closed
- shutdown × Stopped: `wat-scripts/fanout/circuit.wat:543`

---

## WHAT LANDED

- `wat-scripts/census-recoverable-raising.wat` — report-only form walk (D5 copy, filter widened)
- `FINDING-nobody-crashes-on-a-recoverable-error.md`
- `census-run.txt`, `d5-rerun.txt`

No `src/`, no `wat/`, no service script, no arm migrated.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | every new family has BOTH controls | Recv/Send/Call/Connect/ServiceEvent/Stopped: both named and fired. **TrySendOutcome PRESENT = STOP-1** (ABSENT fired). |
| 2 | D5 instrument still passes | A ABSENT, B PRESENT. Numbers drifted 464→466; STOP-5 now PRESENT. sns line 369 drifted. |
| 3 | exemplars per (direction × family) | FINDING §3, seven rows |
| 4 | head/home split | FINDING §2 tables. 2276 total vs 123+84+5+3 live-service |
| 5 | Stopped own bucket | 325 raising, not in 2276 |
| 6 | `:2549` flagged not ranked | unclaimed UNCLASSIFIED, raising ABSENT |
| 7 | floor 5251 / 0 FAIL | Summary **5251 passed**, no ARM.txt |
| 8 | clippy 0 | CLIPPY=0 (`--release --workspace --all-targets -D warnings`) |
| 9 | nextest --no-run | clean |
| 10 | nothing migrated | one census under wat-scripts; 0 src/ wat/ service scripts |
| 11 | direction derived | ServiceEvent::* → service-faces-client; else client-faces-service |
| 12 | headline | a census was taken. Nothing was made tolerant |
