# SCORE — the bracket peer path faces chaos

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `fe6a2af7a` (AMEND mid-strike: the lens is DoS). Did not commit.

Sentence: **A bracket runner must be able to die, stall, and answer badly — on purpose.**

The work-fn already reaches slow and oversize. Die was already on the floor. Suppression is the remaining gap and was **not** built (the null). The deadline is injectable. A 1-runner stall fires `collect-gave-up!` for the first time.

---

## Row 1 — three facts per fault (DoS amendment)

### Die — already used (`probe_dead_runner_loses_one_item`)

| | |
|---|---|
| ServiceEvent | `Lost` (crash) |
| arm | Lost RETRY — `collect-requeue`, drop from `alive` |
| survive? | **yes**, if a survivor remains (`[1,2,3,4]`). Last runner: **no** — `REPORT-GONE` |
| cost | one runner; item re-dispatched. Bounded (crash is prompt) |

### Slow — work-fn waits on `after` then returns

```
arm=Message;got=11;elapsed-ms=202
```

| | |
|---|---|
| ServiceEvent | `Message` |
| arm | Message — result delivered, not re-dispatch |
| survive? | **yes** |
| cost | the nap (at least 200 ms). Completing-slow is bounded by the work-fn. A stall that **never** sends is the next cell |

### Oversize — `O` above `DEFAULT-MAX-MESSAGE-BYTES` (process, 1 runner, 1 item)

```
REPORT-GONE last runner 0 crashed holding item 0: frame exceeded cap
  (message larger than the receiver's max-message-bytes budget)
```

| | |
|---|---|
| ServiceEvent | **`Lost`** — `RecvError::FrameTooLarge` → `classify_peer_error` → `PeerDeath::Lost`. **Not Malformed. Not Rejected.** |
| arm | Lost empty-alive → `collect-report-gone!` (`assertion-failed!`) |
| survive? | **no**. A worker takes down its coordinator. A service would have answered `Rejected` (400-class). |
| cost | bounded (the raise is prompt). The DoS here is death, not a hang |

⭐ **This is the trust asymmetry measured:** `decode_trusted_wire` names the premise; an oversized reply is treated as a crashed peer, not a client error.

### Stall that never sends (before the bound)

`select` of N spawned runners is **unbounded**. `after` cannot join that set (unified Peer vs Thread/Process — STOP-1). A stall of every runner in a pool of 2+ still hangs. That is a DoS even when nothing crashes. Row 4 bounds the 1-peer case.

---

## Row 2 — oversize is not Malformed

Quoted above. FrameTooLarge → Lost → REPORT-GONE. The Rejected arm (`over-budget frame`) did not run. The Malformed arm did not run.

---

## Row 3 — Malformed is unreachable on this path

A type-correct work-fn returns an `O` that encodes and decodes by construction. Byte corruption is not injectable (FINDING §4). `c4026f99e` stays **correct-but-latent**, guarded by the source pin (`malformed_requeues_and_keeps_the_runner_alive`). No corruption injector was invented.

---

## Row 4 — the bound fires

`(:wat::program::collect-deadline-ms)` — handshake shape. One home: `DEFAULT_COLLECT_DEADLINE_MS = 300_000` in `src/intrinsic/program.rs`. Env `WAT_COLLECT_DEADLINE_MS`. Production default unchanged (probe: `300000`). The wat `defn` that held the literal is gone.

1-peer collect uses `recv-by-deadline` (`collect-wait-one`). N-way `select` is still unbounded.

Injected `WAT_COLLECT_DEADLINE_MS=200`, work-fn naps 2000 ms, 1 thread runner:

```
GaveUp waited-ms=200 last=TimedOut (wall-clock bound 200 ms; per-item causes are not carried — the surface has no slot)
```

First `collect-gave-up!` produced by a real stall.

---

## Row 5 — non-vacuity

| probe | fire evidence |
|---|---|
| slow | elapsed-ms=202 ≥ 200; got=11 (the nap ran, then the work) |
| oversize | `frame exceeded cap` in the raise |
| stall | `waited-ms=200` and `wall-clock bound 200 ms` |
| die | `[1,2,3,4]` — survivor did the dead runner's item |

---

## Row 6 — scope wall

Not done: thread-tier *transport* chaos (the nap is a handler park, not a socket fault); lineage-Admin delay; backpressure / healing partition / half-close / byte corruption; `wat/queue.wat` knobs. Hardening (runner-wire caps, 400-class oversized reply, retiring trusted-wire for a remote fleet) is the stone this feeds, not this one. Suppression was not built.

---

## Row 7 — floor

`git diff HEAD` was **not empty** (did not commit). Floor ran on HEAD `fe6a2af7a` plus this tree:

```
 src/check.rs                                     | 14 +++++++
 src/intrinsic/program.rs                         | 52 ++++++++++++++++++++++++
 tests/kernel/probe_bare_recv_outcome_surface.rs  | 17 +++++---
 tests/kernel/probe_dead_runner_loses_one_item.rs | 48 +++++++++++-----------
 wat/bracket.wat                                  | 41 +++++++++++++++++--
 5 files changed, 138 insertions(+), 34 deletions(-)
```

Untracked: five `tests/kernel/probe_bracket_chaos_*` + `probe_bracket_peer_path_faces_chaos.rs`, two scratch-pad measurements.

**First floor RED** — `.floor/2026-09-19T10-34-12Z/`, **not re-run as a green**:

```
     Summary [ 129.320s] 5334 tests run: 5330 passed, 4 failed, 22 skipped
```

Arms (each a different mechanism):

1. `no_loose_string_assert` — eight `contains` sites without `rune:lint(loose-assert)`
2. `purity_mandated_examples` — missing runnable `@example` on the new intrinsic
3. `bracket_recvoutcome_matches_are_fed_by_bare_recv` — pin expected 0 `recv-by-deadline`; collect-wait-one is 1
4. `brackets_four_unreachable_serviceevent_arms_are_counted` — Malformed count 2 (constructor + arm)

Fixed the four; **new** floor:

```
     Summary [ 132.208s] 5334 tests run: 5334 passed, 22 skipped
```

`.floor/2026-09-19T10-39-54Z/` — exit **0**, **no `ARM.txt`**. Count **5329 → 5334** (+5). Not a shrink. CLIPPY=0.

Floor times: slow 0.466 s · oversize 0.490 s · stall 2.265 s.

---

## What landed

- `(:wat::program::collect-deadline-ms)` + `WAT_COLLECT_DEADLINE_MS`
- `collect-wait-one` — 1-peer `recv-by-deadline`; TimedOut → `collect-gave-up!`
- Five kernel probes
- Predecessor pins updated so a constructor earlier in the file is not mistaken for the collect-loop arm

No suppression knob. `wat/core.wat` line count untouched.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | measurement, three facts | die Lost RETRY survives-with-survivor; slow Message survives cost≥200ms; oversize Lost REPORT-GONE **dies**; stall hang is unbounded at N>1 |
| 2 | oversize ≠ Malformed | Lost / frame-cap / REPORT-GONE |
| 3 | Malformed reachability | **unreachable**; pin stays |
| 4 | bound fires | GaveUp last=TimedOut bound 200 ms |
| 5 | non-vacuity | table above |
| 6 | scope wall | named; no suppression; no hardening |
| 7 | floor | red captured then new green; dirty tree quoted; CLIPPY=0 |

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted; the DoS is confirmed, and the residual hole has a named root

### ⭐ THE HEADLINE IS CONFIRMED BY MY OWN RUN: a worker kills its coordinator

```
bracket collect-loop: REPORT-GONE last runner 0 crashed holding item 0:
  frame exceeded cap (message larger than the receiver's max-message-bytes budget)
```

One oversized reply from one worker and the bracket **dies**. The builder's framing —
*"it can be dos'd by its worker fleet with IPC, just like with services"* — is now measured rather
than argued, and the asymmetry is exact: the same fault at a **service** is `Rejected`,
*"a 400-class CLIENT error, NOT a 500-class internal crash"*. At a bracket it is `Lost` →
`REPORT-GONE` → `assertion-failed!`.

⚠ And note **which** variant: `FrameTooLarge` → `classify_peer_error` → **`Lost`**. Not `Rejected`,
not `Malformed`. Row 2's demand was not to conflate them, and it did not.

Slow reproduces too: `arm=Message;got=11;elapsed-ms=201` — a completing-slow worker is survivable
and its cost is the work-fn's own nap.

### ⭐ Row 3 is the null delivered honestly, and it grades one of my commits

`Malformed` is **unreachable** on this path: a type-correct work-fn returns an `O` that encodes and
decodes by construction, and byte corruption is not injectable. So `c4026f99e` — my own
`Malformed` re-dispatch fix — stands **correct-but-latent** behind its source pin. ⛔ **No
corruption injector was invented to make a committed fix look exercised.** That refusal is worth
more than the fix.

### ⛔ THE RESIDUAL DoS, AND ITS ROOT IS A BOUNDARY WE ALREADY DREW

The new bound covers **a pool of one**. `collect-wait-one` uses `recv-by-deadline`; the N-way
`select` is still **unbounded**, so a fleet whose every runner stalls hangs the coordinator for
ever. A reader must not take row 4's success as a general bound.

⭐ **And the reason is structural, not an oversight.** I verified it: `eval_kernel_after` mints
`crate::kernel::spawn::PEER_TYPE_PATH` — a **unified** Peer — while bracket's runners are
`THREAD_PEER_TYPE_PATH` / `PROCESS_PEER_TYPE_PATH`. `eval_peer_select_values` dispatches on the
**first** peer's `type_path`, so a timer cannot join a bracket's select set at all.

> **That is the un-merged half of `one-selectable-set-primitive`.** That stone unified the
> unified-Peer path and left spawn Thread/Process as separate *input types* — correctly, on the
> evidence it had. This is the first measured cost of that boundary: a bracket cannot put a clock
> in its own wait, so it cannot bound a fleet-wide stall.

### Verified

| claim | check |
|---|---|
| deadline injectable, ONE home | ✅ `DEFAULT_COLLECT_DEADLINE_MS = 300_000` (`src/intrinsic/program.rs:190`), env `WAT_COLLECT_DEADLINE_MS`; the wat literal is **gone** (0 × `300000` in `bracket.wat` code). Production default unchanged |
| `src/check.rs` +14 justified | ✅ the new intrinsic's type registration, with the reason written at the site |
| floors | ✅ RED `.floor/2026-09-19T10-34-12Z/` (4 arms, each a different mechanism, captured and **not** re-run as a green) → green `.floor/2026-09-19T10-39-54Z/`, `5334 passed`, no `ARM.txt` |
| scope wall | ✅ no suppression knob, no hardening, queue knobs untouched |

⚠ Two of the four reds were **my own predecessor pins** being too strict
(`bracket_recvoutcome_matches_are_fed_by_bare_recv` expected 0 `recv-by-deadline`;
`brackets_four_unreachable_serviceevent_arms_are_counted` counted a constructor as an arm). They
fired correctly — the code changed under them — and were narrowed rather than deleted.

### What the next stone must carry

1. **Hardening** — a cap on the runner wire and a 400-class disposition for an oversized reply, so
   a worker cannot kill its coordinator. This is now evidenced, not speculative.
2. ⛔ **A fleet-wide stall bound**, which needs a clock that can join a spawn-tier select — i.e. it
   reaches back into the select boundary.
3. Suppression, still unbuilt and still the only primitive a work-fn cannot reach.
