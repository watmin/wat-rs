# BRIEF — the fill poller gets what the drain poller got

Rebuild `:fanout::poll-until-filled*` on the shape `:fanout::poll-until-drained*` already has: **give up
on lack of progress, not on an attempt budget, and say which world it is in.** Fix the unsatisfiable
completion test. Report the overshoot rather than swallowing it. Then audit the siblings.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-fill-poller-gets-what-the-drain-got/DESIGN.md`
2. `docs/excursus/2026/08/001-sns-sqs/we-fixed-one-of-two-pollers/FINDING.md` — the captured red and its
   arithmetic.
3. **`wat-scripts/fanout/circuit.wat:1433-1460`** — the drain poller's header. ⛔ **This is your
   specification, not an inspiration.** It states the property to preserve above all: *"the ONLY path
   returning `""` is the completion test, and every other path is bounded — the stall arm by K, and the
   ceiling arm unconditionally by wall clock regardless of progress."*
4. `wat-scripts/fanout/circuit.wat`, `:fanout::poll-until-drained*` and its constants
   `:fanout::drain-stale-polls` (`:1417`, = 600) and `:fanout::drain-ceiling-ms` (`:1434`,
   = 30000 + 12×pairs). **Mirror these, and state your fill values' reasoning in the SCORE.**
5. **`:fanout::sweep-filled?`** — the defect: `(= (first d) n)`. And `:fanout::poll-until-filled*`
   (`:1363-1387`) — the attempt-bounded loop to replace.
6. `wat-scripts/fanout/circuit.wat:1181` — `depth-of` returns `(visible, unacked, acks)`. **Your progress
   signal is `Σ(visible + unacked + acks)`, free from the sweep already taken.**
7. `wat-scripts/fanout/circuit.wat:2650` — `(poll-until-filled qclients topic n (* n m))`, the caller
   whose `n × m` is the budget being replaced.

**Sketch:**

```
;; progress = Σ(visible + unacked + acks) over qclients. MONOTONE under BOTH fill-first? modes.
;; Σvisible alone is WRONG: with fill-first?=false consumers drain during fill.
poll-filled* [qclients t n start-ns prog-prev stale stale-max rts]
  sweep, box, prog-now                          ← one sweep, exactly as today
  if unread                 -> "filled-unread: …"                        (unchanged, outranks)
  if sweep-filled? >= n and box=0 -> "" , and REPORT the excess if visible > n
  if prog-now > prog-prev   -> recurse, stale = 0
  if stale >= K             -> "filled-stalled: no arrival progress in {K} polls; …"
  if elapsed >= CEILING     -> "filled-timeout: still progressing; stale={s} stale-max={m} …"
  else                      -> sleep 5 ms, recurse with stale + 1
```

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat`. **The compiler and the corpus gate are
the census** — anything they force, make and name.

**STOP-1 — the check must still be able to go RED.** This is the row the drain stone was most afraid of
and it applies verbatim here: a load-tolerant poller that can no longer fail is worse than the
unsatisfiable one it replaces. **Convince yourself, and say in the SCORE how**, that a system which
genuinely stops filling still produces a non-empty verdict. Exercise it — `j=0` gave the drain poller its
witness (`rc=2, drained-stalled`); find the fill equivalent and run it.

**STOP-2 — if `Σ(visible + unacked + acks)` is not monotone, STOP and report the path.** My whole signal
choice rests on it. A redelivery that re-increments `visible` without incrementing anything else, or an
`acks` reset, would break it. **Verify from the code, not from my sentence.**

**STOP-3 — if the sibling audit finds another one-sided fix, STOP and report it before fixing it.** That
would be the fifth instance today, and at that point it is a *class* wanting a gate, not four separate
repairs. The builder rules on that, not you.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it.

**The audit is part of the deliverable, not an extra.** Check `sweep-unread?`, `snapshot-str`,
`sweep-drained?`, and the `n × m` budget at `:2650` for the same one-sided treatment. ⚠ **Report it even
when it finds nothing** — *"checked, the sibling is fine"* is the result that stops this recurring.

**Measure**: `2000 4 3 8192 true 1000` ×3 before and after, interleaved, box quiet. ⚠ The happy path
should be **unchanged** — a successful fill never reaches the give-up path. **A speedup here means you
changed the completion path, which is a finding.** Sequential blocks have twice produced false results in
this campaign (`9f1392630`, `b8f894eca`).

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). `.wat` is edited with an editor —
**not python or sed**. **Read the floor's Summary line, never a piped exit code.** Leave everything
uncommitted.

**Write your SCORE** to
`docs/excursus/2026/08/001-sns-sqs/the-fill-poller-gets-what-the-drain-got/SCORE.md`, graded row by row.
Copy the shape of `docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/SCORE.md`
— the same stone, on the other poller.
