# BRIEF — a wait that SHOULD be bounded (the classification)

⛔ **REPORT-ONLY. Change no code.** Your deliverable is one markdown file.

You are striking one stone in a long campaign to make wat's service-to-service layer an exemplar: no
service may crash or hang on a recoverable error. **Read `DESIGN.md` beside this first** — it carries the
four classes, three controls with known answers, and five trap-doors.

## The work, in one paragraph

A previous census (`wat-scripts/census-waiter-bounds.wat`) found **259 bare `(:wat::kernel::recv …)` sites
with no deadline** — but that measured *whether* a wait is bounded, not *whether it should be*. Three
kinds hide in there: a recv on a **timer** (bounded by construction), a deliberate **park** (blocking is
the feature), and a genuine **unbounded handshake or request/reply** (the defect). Classify every **live**
site — `wat/**` plus `wat-scripts/{queue/sqs,topic/sns-fanout,fanout/circuit}.wat` — into
`TIMER | PARK | BOUND | UNKNOWN`, **from the code**, with evidence per site.

## Why the distinction is load-bearing

- `(recv (after PeerKind::thread (Milliseconds ms) :done))` — the peer **is** a timer. It fires. Bounding
  it would put a timeout on a timeout. → **TIMER**
- `:user::park-receive!` (`wat-scripts/queue/sqs.wat:2053`) takes a `:queue::Queue::Wait`. It is the
  **long-poll**; blocking is the point, and a struck stone (`queue-long-poll`) exists to make it so.
  → **PARK**
- A service's main loop awaiting its next request must block forever — that is what a service *is*.
  → **PARK**
- `wat/service.wat`'s generated `child-main` awaits its owner's one-time startup ship. If the owner is
  slow, the child hangs **and cannot be killed by SIGTERM**. → **BOUND** (and already fixed, see controls)

## How to find the sites

```bash
cd /home/john/work/holon/wat-rs
grep -rn '(:wat::kernel::recv ' --include=*.wat wat/ wat-scripts/queue/sqs.wat \
    wat-scripts/topic/sns-fanout.wat wat-scripts/fanout/circuit.wat
```

⚠ **`recv-by-deadline` is a DIFFERENT primitive** and is already bounded — make sure your pattern does not
catch it (note the trailing space above).

⛔ **Read each site's surrounding code.** The orchestrator's own list came from grepping a census report
and reading enclosing-function names, and **four rows carried the same function name** — so some may be
one function counted repeatedly, or several distinct recvs inside it. **A census row is a pointer, not a
fact.** Re-derive the site count from the code and report the real number.

## The three controls — validate yourself before trusting your own output

| site | must classify as |
|---|---|
| `:fanout::await-timer-ms` (`wat-scripts/fanout/circuit.wat:1208`) and its `sqs.wat` / `sns-fanout.wat` twins | **TIMER** |
| `:user::park-receive!` (`wat-scripts/queue/sqs.wat:2053`) | **PARK** |
| `wat/service.wat`'s `child-main` (`grep -n 'child-main-form'`) | **BOUND**, and note it is **already fixed** — it now calls `recv-by-deadline`, so it should not even appear in your bare-`recv` list. If it does, you are reading stale code. |

⛔ **If any control misclassifies, say so and STOP.** A classifier that cannot sort three known sites
cannot be trusted on the rest. That is a valid, useful outcome — report it.

## Deliverable

Write **`docs/excursus/2026/08/001-sns-sqs/a-wait-that-should-be-bounded/FINDING-the-classification.md`**:

1. **The three controls**, each with its verdict and PASS/FAIL.
2. **A table of every live site**: `file:line` · enclosing defn · **what it is waiting for** (one phrase,
   from the code) · class · one-line reason.
3. **Totals per class**, and the live-site count **re-derived from the code** (say if it differs from 24).
4. **The BOUND list alone**, restated — that is the defect list this stone exists to produce.
5. **The UNKNOWNs**, with what specifically defeated you. ⛔ Never fold an UNKNOWN into another class.
6. **The non-live count** (tests/probes/scratch-pad/docs) so the live/total ratio is visible — a number
   only, do not classify them.

## STOP triggers

1. ⛔ **STOP-1 — a control misclassifies** → say so, stop, report.
2. ⛔ **STOP-2 — you cannot tell PARK from BOUND at a site** → **UNKNOWN**. Mis-sorting a PARK as BOUND
   would break a struck feature (`queue-long-poll`). Guessing costs more than admitting.
3. ⛔ **STOP-3 — you are about to edit a `.wat` or `.rs` file** → don't. Report-only. If a probe is truly
   needed to settle a class, put it in `wat-scripts/scratch-pad/` and say why reading was insufficient.
4. **STOP-4 — the site count differs materially from 24** → report the real number **with the sites**.
   The code wins over the orchestrator's figure.

## Verify before you hand back

- Every site in your table has a **real `file:line` you opened** — not a census row you trusted.
- The three controls are stated with verdicts.
- Totals add up to the re-derived site count.
- `git status --porcelain` shows **only** your new FINDING file.

## Context you may want

- `wat-scripts/census-waiter-bounds.wat` — the census that produced `recv 0/259`, and its own run output at
  `docs/excursus/2026/08/001-sns-sqs/every-waiter-can-bound-its-wait/census-run.txt`.
- `docs/excursus/2026/08/001-sns-sqs/every-waiter-can-bound-its-wait/SCORE.md` — the numbers and why
  `UNKNOWN` is first-class.
- `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md:43` — *"a bare
  `(:wat::kernel::recv peer)` blocks forever and can never return `TimedOut`"*, the substrate fact under
  all of this.
