# FINDING — nobody crashes on a recoverable error (the census)

Drawn `745a4ff24`. Executor: grok, 2026-09-15, branch `sns-sqs`. **No substrate change.**
`git diff --stat -- src/ wat/` is empty. Nothing migrated.

Instrument: `wat-scripts/census-recoverable-raising.wat` (report-only, form-walk).
Copy of `census-malformed-raising.wat`; D5 file **untouched**.
Raw stdout: `census-run.txt` beside this file. D5 re-run: `d5-rerun.txt`.

---

## 0. What this census is

D5 counted raising `RecvOutcome::Malformed` arms. This stone widens the filter to every
recoverable transport variant, both directions, and reports. **A transport fact about a
peer is recoverable. A substrate defect is not.** `RecvOutcome::Stopped` is its own bucket
(the world is stopping, not a peer failed) and is **not** folded in.

Direction is derived from the variant family, not guessed per-site:
`ServiceEvent::*` → `service-faces-client`; everything else recoverable →
`client-faces-service`; Stopped → `shutdown`.

Path list: `find wat wat-scripts tests docs -name '*.wat' | sort` → **1764** files
(D5 was 1753). `wat-tests/` is **out** (same four homes as D5's original). Stated, not walked.

---

## 1. Controls (stated in the census stdout)

Every newly-covered family has a line+variant PRESENT and ABSENT control, except
**TrySendOutcome PRESENT** (STOP-1 — see §6). D5's own controls still pass on the original
instrument.

```
D5-CONTROL-A circuit.wat:544 RecvOutcome::Malformed raising: ABSENT
D5-CONTROL-A sns-fanout.wat:421 RecvOutcome::Malformed raising: ABSENT
D5-CONTROL-B wat/service.wat RecvOutcome::Malformed raising: PRESENT
CONTROL RecvOutcome PRESENT stdio.wat:221 Lost: PRESENT
CONTROL RecvOutcome ABSENT circuit.wat:544 Malformed: ABSENT
CONTROL SendOutcome PRESENT probe_a_peer_remembers_its_address.wat:44 Closed: PRESENT
CONTROL SendOutcome ABSENT circuit.wat:3873 Closed: ABSENT
CONTROL TrySendOutcome PRESENT: NONE-IN-CORPUS  (STOP-1)
CONTROL TrySendOutcome ABSENT service.wat:2584 Closed: ABSENT
CONTROL CallOutcome PRESENT circuit.wat:1015 Malformed: PRESENT
CONTROL CallOutcome ABSENT probe_a_fired_deadline_hands_back_a_live_peer.wat:65 Lost: ABSENT
CONTROL ConnectOutcome PRESENT circuit.wat:414 Refused: PRESENT
CONTROL ConnectOutcome ABSENT probe-a-dying-client-kills-the-service.wat:109 Refused: ABSENT
CONTROL ServiceEvent PRESENT bracket.wat:626 Closed: PRESENT
CONTROL ServiceEvent ABSENT circuit.wat:3857 Closed: ABSENT
CONTROL RecvOutcome::Stopped PRESENT circuit.wat:543: PRESENT
CONTROL RecvOutcome::Stopped ABSENT bracket.wat:53: ABSENT
FLAG service.wat:2549 ServiceEvent::Lost raising-rows=ABSENT unclaimed=PRESENT
STOP-5 unclaimed let/do/if/macro body: PRESENT — do not quote raising-arms as complete
```

Line+variant is load-bearing: circuit.wat:543 (raising Stopped) and :544 (string Malformed
and string TimedOut) share a physical line. A line-oriented pass without the variant
column would have failed CONTROL A.

### D5 instrument re-run (unchanged)

```
CONTROL-A circuit.wat:544 raising-Malformed: ABSENT
CONTROL-A sns-fanout.wat:369 raising-Malformed: ABSENT
CONTROL-B wat/service.wat raising-Malformed: PRESENT
STOP-5 unclaimed let/do/if/macro body: PRESENT
form-tree keywords=495; match-arms=492; raising-arms=466; unclaimed-arms=26
```

D5 SCORE (2026-09-13) cited 482/479/464/15 and STOP-5 ABSENT. The instrument still
passes its two controls. The numbers drifted with the corpus (+11 files, +2 raising
Malformed, +11 unclaimed). ⚠ sns-fanout CONTROL A is still hardcoded at **line 369**;
the migrated arm has drifted to **line 421**. 369 is ABSENT vacuously (no Malformed
arm there). The widened census pins 421. CONTROL B's cited line 3281 has drifted to
**3524**; D5's check is file-level so it still PRESENT.

Cross-check: widened census `RecvOutcome::Malformed` raising = **466**, matching the
D5 re-run exactly.

---

## 2. Recoverable population

```
recoverable keywords=3323; match-arms=3304; raising-arms=2276; unclaimed-arms=1028
STOPPED-BUCKET keywords=493; match-arms=492; raising-arms=325; unclaimed-arms=167
```

All 2276 recoverable raises are `assertion-failed!`. `raise!`/`panic!` recognised; 0 used.

STOP-5 is PRESENT: unclaimed bodies include `let`/`do`/`if`. Raising-arms is therefore
**not complete** for families whose fatal work sits inside those forms. The walker does
not guess. `service.wat:2549` is the named case (body is `do`).

### by family (raising)

| family | raising | direction |
|---|---|---|
| RecvOutcome | 1471 | client-faces-service |
| ConnectOutcome | 795 | client-faces-service |
| CallOutcome | 5 | client-faces-service |
| ServiceEvent | 3 | service-faces-client |
| SendOutcome | 2 | client-faces-service |
| TrySendOutcome | 0 | client-faces-service |

### by variant (raising)

| variant | n |
|---|---|
| RecvOutcome::Malformed | 466 |
| RecvOutcome::TimedOut | 377 |
| RecvOutcome::Lost | 317 |
| RecvOutcome::Closed | 311 |
| ConnectOutcome::{Refused,Rejected,Failed} | 265 each |
| CallOutcome::Malformed | 2 |
| CallOutcome::{Lost,Closed,DeadlineFired} | 1 each |
| SendOutcome::{Lost,Closed} | 1 each |
| ServiceEvent::{Closed,Lost,Malformed} | 1 each |
| TrySendOutcome::{Lost,Closed,WouldBlock} | 0 |

### by direction (raising)

| direction | n |
|---|---|
| client-faces-service | 2273 |
| service-faces-client | 3 |

The service-facing half of "nobody crashes" is **three raising arms**, all in
`wat/bracket.wat`'s collect-loop. The defservice serve-loop's `ServiceEvent` arms
are unclaimed (see §5).

### by head / home (raising)

| head \ home | wat | wat-scripts/service | scratch-pad | tests | docs | other |
|---|---|---|---|---|---|---|
| **defservice** | 18 | **51** | 6 | 66 | 0 | 28 |
| defn | 55 | 63 | 268 | 1365 | 16 | 273 |
| deftest | 0 | 0 | 0 | 36 | 3 | 0 |
| **other** | **28** | 0 | 0 | 0 | 0 | 0 |
| defsurface | 0 | 0 | 0 | 0 | 0 | 0 |

Totals: defservice 169 · defn 2040 · deftest 39 · other 28 · defsurface 0.
homes: wat 101 · wat-scripts/service 114 · scratch-pad 274 · tests 1467 · docs 19 · other 301.

D5's lesson repeats: **2276 is a population, not a work list.** tests+scratch-pad+docs+other
= 2061 of 2276. Dying loudly is correct in a probe.

### family × home (the cut)

| family | wat | wat-scripts/service | tests | scratch-pad | other | docs |
|---|---|---|---|---|---|---|
| RecvOutcome | 68 | 55 | 982 | 148 | 214 | 4 |
| ConnectOutcome | 30 | 54 | 483 | 126 | 87 | 15 |
| CallOutcome | 0 | 5 | 0 | 0 | 0 | 0 |
| ServiceEvent | 3 | 0 | 0 | 0 | 0 | 0 |
| SendOutcome | 0 | 0 | 2 | 0 | 0 | 0 |
| TrySendOutcome | 0 | 0 | 0 | 0 | 0 | 0 |

Live-service raising (wat + wat-scripts/service): RecvOutcome **123**, ConnectOutcome **84**,
CallOutcome **5**, ServiceEvent **3**, SendOutcome **0**, TrySendOutcome **0**.
`defservice × wat-scripts/service` = **51** (all recoverable families together; D5's
Malformed-only cell is now 17).

---

## 3. Exemplars per (direction × family)

Quoted from raising rows unless STOP-1 / unclaimed is the finding.

| direction × family | exemplar | raises |
|---|---|---|
| client-faces-service × RecvOutcome | `wat/kernel/services/stdio.wat:221` Lost | assertion-failed! |
| client-faces-service × SendOutcome | `tests/comms/probe_a_peer_remembers_its_address.wat:44` Closed | assertion-failed! |
| client-faces-service × TrySendOutcome | **none raising.** `wat/service.wat:2584` Closed | nil (unclaimed) |
| client-faces-service × CallOutcome | `wat-scripts/fanout/circuit.wat:1015` Malformed | assertion-failed! |
| client-faces-service × ConnectOutcome | `wat-scripts/fanout/circuit.wat:414` Refused | assertion-failed! |
| service-faces-client × ServiceEvent | `wat/bracket.wat:626` Closed | assertion-failed! |
| shutdown × RecvOutcome::Stopped | `wat-scripts/fanout/circuit.wat:543` Stopped | assertion-failed! |

Further raising exemplars the builder can open:

- RecvOutcome Closed: `stdio.wat:226`. TimedOut: `stdio.wat:227` (same physical line as Malformed).
- SendOutcome Lost: `probe_a_peer_remembers_its_address.wat:46`. **Zero raising SendOutcome
  arms in `wat/` or `wat-scripts/service`** — those bodies continue, return nil, or a string.
- CallOutcome Lost/Closed/DeadlineFired raising: `circuit.wat:2391/:2393/:2389` (a helper
  `defn`, not the worker). The worker's Lost/Closed/DeadlineFired at `:982/:991/:1000` are
  `UNCLASSIFIED` (`let` that redials).
- ConnectOutcome Rejected/Failed: `circuit.wat:416/:418` (same init as Refused).
- ServiceEvent Lost/Malformed raising: `bracket.wat:632/:642` (collect-loop). **Not** the
  serve-loop.

---

## 4. FLAG — `wat/service.wat:2549` is not rankable

```
wat/service.wat:2549:23  head=other  home=wat  variant=ServiceEvent::Lost
  family=ServiceEvent  direction=service-faces-client  raises=UNCLASSIFIED
```

It **appears** (unclaimed). It is **not** in raising rows: body head is `do`, and STOP-5
forbids looking inside. Human reading of that `do`: `assertion-failed!` then dead
`remove-at` recur. `FINDING-the-client-vanished-arms-are-unreached.md` already measured
that nothing known reaches this arm. Which kind of unreachable is unanswered. **Not
ranked as work.**

Siblings, also unclaimed, for the same serve-loop:

- `:2539` ServiceEvent::Closed `raises=list` (evict + recur — the correct arm)
- `:2559` ServiceEvent::Malformed `raises=:wat::core::match` (reply Failed, keep serving)

`call-by-deadline` at `:4258/:4262/:4274` is also `UNCLASSIFIED` (`if`). Those arms
construct a `CallOutcome`, they do not raise.

---

## 5. RecvOutcome::Stopped — own bucket, not folded

325 raising, 167 unclaimed. PRESENT `circuit.wat:543`, ABSENT `bracket.wat:53` (nil).
The builder rules whether shutdown paths belong in a later migration. This census
must not sweep them into recoverable.

---

## 6. STOP-1 — TrySendOutcome has no raising arm in the tree

15 match-arms, 0 raising, in the four homes **and** in `wat-tests/` (corpus grep for
`TrySendOutcome::{Lost,Closed,WouldBlock}` next to `assertion-failed!` is empty).

Bodies are `nil` (serve-loop evict, `service.wat:2583–2585`) or probe labels
(`string` / `format`). ABSENT control fires. PRESENT cannot be named: there is no
raising arm the walker could be asked to find.

The TrySendOutcome raising count **0** is therefore a number the walker produced
and a grep corroborated, but it is not a PRESENT-controlled finding. Report it as
such. Do not migrate it on the strength of this census.

SendOutcome is the near-miss: only two raising arms, both in one test probe; live
`wat/` / `wat-scripts/service` already tolerate Closed/Lost (continue or nil).

---

## 7. What this census does not claim

- **Not a work list.** 2276 is a population. The builder sequences.
- **Not "live service = 51."** That cell is all recoverable families inside
  `defservice × wat-scripts/service`. RecvOutcome Malformed-only is 17 (D5 re-run).
- **Not wat-tests.** Same four-home list as D5. `wat-tests/service-deferred-reply.wat`
  raises on SendOutcome::{Closed,Lost}; those rows are not in this walk.
- **Not complete where STOP-5 is PRESENT.** A `let`/`do`/`if` whose last expr raises
  is unclaimed, not raising. `:2549` is the proof.
- **Not a migration.** Nothing was made tolerant.

Reproduce:

```
find wat wat-scripts tests docs -name '*.wat' | sort \
  | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
  | ./scripts/capped.sh --limit 8g ./target/release/wat ./wat-scripts/census-recoverable-raising.wat
```
