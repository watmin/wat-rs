# BRIEF — the instrument fits the question

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`.
Read `DESIGN-the-instrument-fits-the-question.md` first — it carries the census, the mechanism, and
the affirmative cuts.

## THE WORK

The topic and circuit test helpers use a **destructive read to prove a negative**. `after-drain`
asks *"is anything here?"* with `take-one`, which consumes the message and hides it for ~1000 s — so
when the answer is *yes*, the check eats it and the `wait-pending` spin two lines later waits forever
for the thing it just removed. That is the live race and it produced a 30 s timeout with an empty ARM.

Give each question its instrument: **absence → `stats`; presence → one `Queue/receive` with
`:wait (Wait::UpTo …)`**. Where no wire event exists (cross-queue, conjunctions, waiting for zero),
the poll stays — **bounded, and reporting what it last saw.** Then make the names and the failure
sentinels stop lying.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat`** — **run it first.** It is the
   mechanism, the acceptance criterion, and the worked reference all at once. Its `:rr::run` cells
   reproduce the hang (`gap=300 → SPINS-FOREVER`); its `:rr::run-lockstep` cells are **the shape you
   are building**, already working (`gap=300 → delivered; raced=yes-and-VISIBLE`), already on
   Stone B's `:wait (Wait::UpTo …)`.
2. **`wat-scripts/topic/sns-fanout.wat:462-482`** — `q-depth`, `wait-inflight`, `wait-pending`.
   Note `q-depth` returns `(Tuple 1 1)` on `Lost`/`Closed`/malformed — a value that **satisfies both
   wait predicates**, so a dead peer reads as work present.
3. **`wat-scripts/topic/sns-fanout.wat:484-499`** — `take-one`. `:visibility-ns 1000000000000` is
   **hardcoded inside**, so no call site can see the 1000 s hold it commits to.
4. **`wat-scripts/topic/sns-fanout.wat:790-802`** — `:user::refused-is-retried`, the site that hung.
   `:793` waits on **inbox** then takes from **subq** (cross-queue — keep as a bounded poll);
   `:796` is the destructive absence check; `:798-799` is `wait-pending` + `take-one` on the same
   queue (collapse to one blocking receive).
5. **`wat-scripts/topic/sns-fanout.wat:836-846`** — `:user::stalled-does-not-stall`. `:838-839`
   collapses; `:840` `stalled` is an absence check.
6. **`wat-scripts/fanout/circuit.wat:480-525`** — `fully-drained?`, `wait-drained`,
   `wait-pending-zero`. **`:486-488` is the model comment** — it declares its own unboundedness and
   explains the third drain term. That comment is what `wait-pending`/`wait-inflight` never had.
7. **`wat-scripts/queue/sqs.wat:64-72`** — `StatsResponse`. The comment at `:69` already says
   *"pending = visible (not yet received). in-flight = received, not yet acked"* — the definition
   contains the better names.
8. **`wat-scripts/topic/sns-fanout.wat:143-147`** — a comment saying *"Do not invent a depth we did
   not read"* immediately above a line returning `Ok 1 0`, **on the redial-recovery path**.
9. **`wat-scripts/topic/sns-fanout.wat:439-442`** — `ticks-of` returns **`-1`** on failure. The file
   already knew the honest convention, four lines from the ones that don't use it.
10. **`docs/arc/2026/06/278-rules-engine/BRIEF-278-a-liveness-bound-only-catches-a-hang.md`** — the
    arc's own three-role taxonomy for bounds: **LIVENESS** (raise; only a hang may trip it) /
    **WINDOW** (never raise; it *is* the scenario) / **NEGATIVE ASSERTION** (coupled to its window).
    ⛔ `:vis-ns 200000000` on the refusal probe is a **WINDOW** — widening it deletes the scenario.
11. **`wat-scripts/fixes/wait-ns-to-wait.wat`** — the freshest recorded codemod, for the renames.

## SKETCH

```wat
;; presence — one call, arrives on the wire, nothing to eat
(:demo::receive-blocking subq "q0" (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 500)))

;; absence — non-destructive, cannot consume what it observes
(:wat::core::first (:demo::q-depth subq))        ;; 0 == nothing visible

;; the irreducible poll — bounded AND reporting
(:wat::core::defn :demo::poll-until-unacked
  [q <- :queue::Queue  attempts <- :wat::core::i64] -> :wat::core::String
  ;; returns "" on success, or "unacked-never-rose: last=<v>/<u> attempts=<n> elapsed=<ms>"
  ...)
```

## STOP TRIGGERS

1. **You are about to widen `:vis-ns 200000000` or the 350 ms nap.** Those are WINDOWs — the
   scenario, not a bound. Widening deletes the race instead of surfacing it. STOP.
2. **A bounded wait can only say "timed out".** That is the empty ARM again. It must name what it
   last saw. STOP and report what blocked it.
3. **You cannot collapse a `wait-*` + `take-one` pair** into one blocking receive at `:798` or
   `:838`. STOP and report which and why — that pair is the stone.
4. **You are about to touch `accept!`, `face-start-tw`, `nap-ms`, or merge the `do-receive`
   helpers.** Stone D2. STOP.
5. **The circuit's invariant moves.** `distinct=8000; dup=0`. Any change is a finding — capture it,
   do not tune it away.
6. **A floor test other than the known race goes red.** Capture whole, name the arm, do not re-run.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop — three riders on this arc died that way.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ `probe_async_publish::refused_subscriber_is_retried_not_dropped` currently passes **by luck**. If
your change makes it fail loudly with a message naming the race, **that is a success, not a
regression** — report it as such and do not weaken the assertion to get green.

Leave your work uncommitted. Prior comparable result for shape: `SCORE-the-wait-names-its-verb.md`.

## REPORT

- the reproducer's cells, before and after — the `gap=300` cell must no longer be able to stall
- **the induced-race result**: with a 300 ms gap, does the test fail loudly or pass? Which, and the
  message verbatim
- a bounded wait forced to expire, with **what it reported**
- a dead peer mid-wait: the helper must report a failure, not a depth
- the grep showing no `receive` proves a negative
- the circuit: `total`, `distinct`, `dup`, five runs, publish ms
- the codemod's own census count, before applying
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas, especially where this brief did not match the disk.** My last four censuses
  were each wrong in a different way; treat mine as a hypothesis and the finder as the fact.
