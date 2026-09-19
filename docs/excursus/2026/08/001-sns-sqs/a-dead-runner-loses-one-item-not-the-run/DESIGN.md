# A dead runner loses one item, not the run

**The crusade's real target in `bracket.wat`.** Excursus `001-sns-sqs`. Read first:

- `../a-momentary-failure-is-not-fatal/DESIGN.md` — **drawn 2026-09-10, still UNSTRUCK.** The
  governing taxonomy (RETRY / REPORT-FINAL / REPORT-GONE). This stone is its first real consumer.
- `../the-bracket-runs-on-the-queue/SCORE.md` + its regrade — and `632335c55`, which found that
  the ten arms that stone targeted are **dead**, and that the live ones are elsewhere.
- `tests/kernel/probe_bare_recv_outcome_surface.rs` — the reachability pins this rests on.

## The sentence

> **One runner dying should cost one item, not the whole bracket.**

## What is actually there — measured 2026-09-19, not assumed

`:wat::bracket::collect-loop` (`wat/bracket.wat:620`) drives the pool. It returns
`(Vector :- [(Tuple :- [:wat::core::i64 O])])` and matches **eight** `ServiceEvent` arms from
`(:wat::kernel::select peers)`. **Seven of the eight raise.**

| arm | disposition today | emitters in the select path |
|---|---|---:|
| `Message` | continue — the only non-raising arm | 3 |
| `Closed` | ⛔ `assertion-failed!` | 4 |
| `Lost` | ⛔ `assertion-failed!` | 2 |
| `Malformed` | ⛔ `assertion-failed!` | 1 |
| `Rejected` | ⛔ `assertion-failed!` | 1 |
| `Shutdown` | ⛔ `assertion-failed!` | 5 |
| `Connection` | ⛔ `assertion-failed!` | 2 |
| `Admin` | ⛔ `assertion-failed!` | 2 |

⭐ **Every one of the eight variants is constructed by the select path** (counted in
`src/runtime.rs`'s two `SELECT_EVENT_TYPE` impls). So these are not exhaustiveness filler — unlike
the ten `RecvOutcome` arms the previous stone chased, which genuinely cannot fire. **Whether each is
reachable *in a bracket runner pool specifically* is a different and unmeasured question** — see
row 1. `Connection` / `Admin` are service-shaped events and may be structurally impossible here;
their own panic strings hint at it (*"select has no self-peer"*).

⭐ **And the recovery data already exists.** `holding` is a per-runner vector of the item index each
runner is currently working (`holding-set`, `holding-phrase`, `bracket.wat:565`/`:578`). It exists
today **only so the panic message can say what was lost**. The same fact is what makes re-dispatch
possible: when runner *k* dies, `holding[k]` is the item to hand to a survivor.

## The work

### 1. Per-arm reachability, in a bracket pool

For each of the seven raising arms: can it reach `collect-loop`, given that `peers` is a vector of
runner peers with no listener and no admin channel? ⛔ **Answer per arm with evidence, not prose.**
An arm that is structurally impossible here is a legitimate `assertion-failed!` (a substrate
violation, not a runtime failure) and should say so at the site. An arm that is reachable is a live
ungraceful path and belongs in row 2.

⚠ Emitted-somewhere ≠ reachable-here. That conflation is exactly what made the previous stone chase
dead code.

### 2. Place each reachable arm in the taxonomy

| | |
|---|---|
| **RETRY** | `Closed` / `Lost` — the runner is gone, but **its item is known** (`holding[idx]`). Re-dispatch to a surviving runner. This is the stone's whole point. |
| **REPORT, FINAL** | `Malformed` — a reply that did not decode. ⚠ Determine whether the defect is the ITEM or the RUNNER before deciding; the governing DESIGN's contract is that `Malformed` is **never retried**, and this stone inherits that rather than re-deciding it. |
| **REPORT, GONE** | no runners left to re-dispatch to |

Bounded by **wall clock, never an attempt count**, and the report names which bound it hit — the
governing DESIGN's ruling #1, and the same rule the queue runner already follows.

### 3. ⛔ The surface question — name it, do not smuggle it

`(:wat::bracket::map …)` returns `(Vector :- [O])`. There is **no room for a per-item failure
value**, which is why every arm raises. That constraint already bit the previous stone: its row 2
(surface byte-identical) and row 3 (arms become values) **could not both be satisfied**, and nobody
noticed until the work was done.

So this stone's scope is deliberately drawn to **not need** the surface change:

- **RETRY loses nothing** — a re-dispatched item still produces its `O`, so `(Vector :- [O])` is
  still exactly right. ⭐ **This is why `Closed`/`Lost` are the target and not `Malformed`.**
- If, after row 1, the only honest treatment of a reachable arm needs a value the surface cannot
  carry, ⛔ **STOP AND REPORT IT.** Do not widen `map`'s return type in this stone. An outcome-typed
  bracket surface is a separate stone with a builder ruling attached, because it changes every
  caller.

### 4. A control that can redden

⛔ Judged by mutation, and the bar is the one `632335c55` set after the last control failed it:
**kill the mechanism and show the test go red.** Concretely: a bracket over N items where one runner
is killed mid-flight must (a) still return all N results, and (b) go RED if the re-dispatch is
removed. A test that only passes proves nothing.

⚠ **Non-vacuity:** prove the runner actually died in the passing run (the fault fired), the way the
queue control asserts `recv-drops>0`. A green because nothing broke is the failure mode here.

## Scope wall

⛔ Do not touch the ten dead `RecvOutcome` arms — `probe_bare_recv_outcome_surface.rs` pins them as
unreachable and explains why. Do not strike stone 1b. Do not change `map` / `each` signatures. Do
not touch the queue path (`0d8bada7b`, `dac31ac03`).

## The four questions

- **Obvious** — yes: a worker dying should not lose the other 999 items, and the pool already
  records what the dead worker held.
- **Simple** — ⚠ genuinely unclear until row 1. Re-dispatch inside an existing accumulator loop is
  small; a re-dispatch that must also re-establish a dead peer may not be. Score it honestly.
- **Honest** — the RETRY arms are made recoverable and the rest are **classified and stated**, not
  silently left raising. Row 1 is what keeps this from being a guess.
- **Good UX** — `(map locus items work-fn)` unchanged; a dead runner costs a re-dispatch instead of
  the run.
