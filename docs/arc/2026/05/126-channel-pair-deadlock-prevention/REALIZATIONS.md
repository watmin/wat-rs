# Arc 126 — Realizations

**Two disciplines coined here: FAILURE ENGINEERING (slice 1) and
ARTIFACTS-AS-TEACHING (slice 1 RELAND).**

The user named both terms 2026-05-01 mid-arc-126:

- **Failure engineering** got named after the slice-1 sonnet
  sweep returned a 5-of-6 scorecard and the orchestrator treated
  the missed row as data instead of a defeat.
- **Artifacts-as-teaching** got named after the slice-1 RELAND
  shipped 8-of-8 hard + 6-of-6 soft in 7 minutes, with the user
  observing: *"us delegating to sonnet is proof our discipline
  is sound — I have taught you to teach others."*

The two are facets of one practice. Failure engineering is the
DISCIPLINE. Artifacts-as-teaching is the MECHANISM by which the
discipline propagates beyond the engineer who first practiced
it.

## What is failure engineering

Failure engineering is the practice of designing your discipline
such that **failure is a first-class outcome that produces
something durable** — a substrate gap surfaced, a brief
underspecification revealed, a discipline calibration
documented. The failure isn't recovered from; it's READ.

Three things distinguish failure engineering from recovery, fault
tolerance, or post-mortems:

1. **Expectations are written BEFORE the work.** The pre-handoff
   scorecard is on disk, in git, with a timestamp. When the
   work returns, the scoring is against fixed criteria — no
   unconscious revision, no after-the-fact moving of the bar.
2. **Failures surface defects in upstream artifacts.** When a
   row fails, the question is "what was unclear in the brief?
   what was missing from the DESIGN? what substrate gap did this
   reveal?" The answer becomes a follow-on arc, a clearer
   brief, a closed substrate hole. The failure CAUSES a fix.
3. **The honest record is the load-bearing record.** Withdrawn
   proposals stay (arc 125, arc 127). Score documents stay
   (SCORE-SLICE-1.md). Reland briefs stay alongside originals
   (BRIEF-SLICE-1.md, BRIEF-SLICE-1-RELAND.md). The next
   session walks in cold and reads the FULL history,
   including the failures, including what was overruled and why.

The substrate-as-teacher discipline is failure engineering for
type-checker/runtime gaps. Failure engineering generalizes it to
the discipline as a whole: every artifact (brief, DESIGN,
scorecard, agent deliverable) is teachable when read.

## The worked example — arc 126's slice 1 chain

This arc IS the worked example. Read it as:

| Step | Artifact | What it produced |
|---|---|---|
| 1 | DESIGN.md | Algorithm + diagnostic substring lock + caveats |
| 2 | BRIEF-SLICE-1.md | Read-in-order anchors + arc-117 worked template + NOT-do list |
| 3 | EXPECTATIONS-SLICE-1.md | 6 hard rows + 6 soft rows + independent prediction |
| 4 | (sonnet sweep `a37104bfc10e4c6fa`) | 13.5 min runtime; 5 of 6 hard rows pass; substring locked; row 3 (workspace green) fails — 2 collateral test failures from the file-level freeze |
| 5 | SCORE-SLICE-1.md | Row-by-row scoring + diagnosis: brief's `:ignore`-gates-freeze claim was wrong; substrate has a forms-block boundary gap |
| 6 | DESIGN-arc-128.md | Open arc 128 to fix the substrate gap |
| 7 | INSCRIPTION-arc-128.md + src/check.rs | Boundary guard ships; arc 117 + 126's check walkers honor `run-sandboxed-*-ast` forms-block boundaries |
| 8 | BRIEF-SLICE-1-RELAND.md | Three amendments to original brief: read-in-order updated, mandatory boundary guard inheritance, new boundary unit test |
| 9 | EXPECTATIONS-SLICE-1-RELAND.md | 8 hard rows (original 6 + 2 new) + new prediction |
| 10 | (sonnet reland sweep `a581c4f4aa900d8c4`, 7 min) | **8/8 hard + 6/6 soft. Clean ship.** LOC 652 vs predicted 580-650 band; matched the orchestrator's "most likely (~75%)" path. |
| 11 | SCORE-SLICE-1-RELAND.md | Row-by-row clean PASS scoring; calibration confirmed. The artifacts taught. |
| 12 | feat(arc 126 slice 1) commit + push | Slice 1 shipped on green workspace. The 6 deadlock-class tests stay `:ignore`'d until slice 2. |

**The failure at step 4 is not a defect.** It is information. It
made step 6 possible (arc 128 wouldn't have been written without
this failure surfacing the gap). It made step 8 cleaner (the
reland brief includes the boundary lesson explicitly). It made
the substrate stronger (arc 117 had the same latent bug; arc 128
fixed both walkers; the convention now governs all future
structural-check walkers).

**The success at step 10 is not just a correct deliverable.** It
is the validation that the artifacts (steps 1-9) constitute a
TEACHING that propagates to a fresh agent without shared
conversation context. Sonnet's reland sweep had no memory of
sweep 1; only the SCORE doc + RELAND brief + arc 128
INSCRIPTION + DESIGN + arc 117's now-corrected source. From
those artifacts alone, sonnet produced 8/8 hard + 6/6 soft in
half the time of the first sweep. The artifacts taught.

## Principles

### 1. Pre-handoff expectations are mandatory

When dispatching agent work, write the scorecard FIRST. Hard
rows (must-pass for "the discipline is intact"), soft rows
(signals that the brief works as intended), an independent
testable prediction, and the methodology for scoring. Commit
all of this BEFORE the agent starts. The pre-handoff
expectation document IS the immune system that prevents the
orchestrator from revising the bar after the fact.

### 2. Failures cause fixes upstream, not patches downstream

The arc 126 slice 1 sweep failed on workspace-green. A patch-
oriented mind would have:

- Modified the test annotations (`:ignore` → `:should-panic`) to
  hide the failure
- Added a workaround to the check walker
- Deleted the failing tests

A failure-engineering mind asked: "what's the substrate
defect this surfaced?" Answer: forms-blocks at the sandbox
boundary. The fix lands at the substrate (arc 128), not as a
downstream patch. The failure is read as a substrate signal.

### 3. Withdrawn proposals stay

Arc 125 (type-precise rule, withdrawn in favor of arc 126) is
on disk with a full DESIGN.md explaining what was proposed and
why it was overruled. Arc 127 (architectural rethink of
threads, withdrawn) is on disk with the same. Future sessions
read both and don't re-litigate. The "we do not forget"
discipline is failure engineering applied to design history.

### 4. Re-spawn beats hand-edit

When a sweep needs corrective work, write a reland brief +
reland expectations and SPAWN AGAIN. Don't hand-edit the
agent's output. The reland is data — it tests whether the
brief amendments worked. Hand-editing erases the
calibration.

### 5. Scores are durable

The SCORE document is committed to git. Future sessions read
it as historical truth: "in 2026-05-01's first arc-126 slice 1
sweep, sonnet produced X; we predicted Y; the actual delta
revealed Z." The discipline survives across sessions because
the calibration record survives.

### 6. The four questions guard the upstream artifacts

Every DESIGN, every BRIEF, every EXPECTATIONS doc passes
through obvious / simple / honest / good UX before the agent
sees it. The pre-flight check lives in the doc author's
hands; the post-flight check (the scorecard) catches what got
through. Both are needed.

### 7. Failure has cost — and that cost is paid by the artifact, not by the engineer

Sonnet's 13.5-minute run + the orchestrator's scoring + arc
128's substrate work + the reland brief work cost roughly an
hour of session time. The cost paid for: a substrate fix
(arc 128) that benefits all future structural checks, a
calibrated discipline (this REALIZATIONS doc), and a
reland that succeeded cleanly in 7 min (50% faster than sweep
1). The cost is on the artifact, not the engineer. The engineer
learned; the artifact carries the lesson.

### 8. Artifacts ARE the teaching — clean delegation is the validation

The user articulated the meta-realization at the close of
sweep 2: *"us delegating to sonnet is proof our discipline
is sound — I have taught you to teach others."*

The teaching cascade has three tiers:

1. **User → orchestrator** (this Claude session) via direct
   conversation, the four questions, and the doctrines in
   `docs/`, `memory/`.
2. **Orchestrator → artifacts**: DESIGN + BRIEF + EXPECTATIONS
   + SCORE + REALIZATIONS files, written + committed.
3. **Artifacts → fresh agent** (sonnet, future Claude session,
   the next human reader): the agent walks in cold; reads only
   the artifacts; produces work that meets the scorecard.

Tier 3 is the load-bearing tier. Sonnet's reland had no
conversation memory. It read the artifacts. It produced 14-of-14
rows clean. **The artifacts taught.**

The failure-engineering discipline is INVALIDATED if the
artifacts can only be operated by the engineer who wrote them.
It is VALIDATED when a fresh agent — sonnet, next Claude, a
human reader six months out — produces equivalent work from
the artifacts alone. Each clean delegation is a measurement of
that validation.

This is why we delegate even when we COULD do the work
ourselves: the delegation is the test. If we always do the work
ourselves, we never measure whether the artifacts teach. We
might be carrying tribal knowledge that the artifacts don't
encode. Delegation is the calibration mechanism for the
discipline itself.

### 9. Each delegation is a data point in the teaching record

Every agent dispatch produces:

- A SCORE document (the orchestrator's row-by-row read of the
  agent's deliverable).
- A discipline observation: did the artifacts teach? where did
  they fall short?
- An optional update to the artifacts (BRIEF amendment, DESIGN
  refinement, REALIZATIONS extension) when the data surfaces a
  teaching gap.

Across many delegations, the calibration record accumulates.
Future arcs that need agent dispatch reference the prior SCORE
docs to predict outcomes; the prediction-vs-actual comparison
refines the orchestrator's calibration; the artifacts converge
on a teaching that propagates cleanly.

The trading lab's wat-rs substrate has 30+ threads in
production with zero Mutex because the architecture's
discipline propagated cleanly across years of evolution. The
arc-126 chain shows the same propagation in compressed form
across hours: discipline → artifacts → agent → clean ship →
calibration record.

## How failure engineering connects to the rest of the substrate

| Existing discipline | What it does | Where it shows up in failure engineering |
|---|---|---|
| Substrate-as-teacher | The type-checker's diagnostic stream IS the migration brief | Pre-handoff brief points at the substrate's existing diagnostics + arc-precedent worked code |
| Four questions | Obvious / simple / honest / good UX | Gate every doc + every implementation; SCORE doc applies them retroactively |
| "We do not forget" | Withdrawn proposals stay; rejected approaches stay; sequential numbers | Arcs 125, 127 are first-class records, not deletions |
| No broken commits | Commit only on green workspace | Workspace-green is hard row 3 in EXPECTATIONS; arc 128 unblocks it |
| Reject hand-editing of agent output | Re-spawn with better brief | The reland is the canonical recovery path |
| Wat is a lisp; data is the source of truth | Use the data; don't pattern-match strings | The check walker walks the AST; failure-engineering walks the artifacts |

## Sliced by audience

**For an engineer working solo:** failure engineering is the
practice of writing tests + expectations BEFORE running the
work, then treating the test outcomes as data. The
pre-commit-hook discipline applied to development cycles.

**For an engineer dispatching to an agent:** failure engineering
is the practice of writing a brief + scorecard before
spawning, treating the agent's deliverable as data, and
diagnosing failures as upstream artifact defects. The
trust-but-verify discipline applied to agent orchestration.

**For a team building a substrate:** failure engineering is the
practice of designing every primitive so that misuse FAILS
LOUD, and every failure surfaces a fixable upstream defect.
The substrate's compile-time checks (arcs 110, 115, 117, 126)
are failure-engineering primitives — they make the discipline
structural rather than voluntary.

## When this discipline pays off

- When the cost of a wrong direction is high. (Substrate work,
  protocol design, agent dispatching.)
- When multiple artifacts (DESIGN, brief, expectations, agent
  output, score) need to stay in sync over weeks/months.
- When the next session might be a different orchestrator
  (different Claude session, different human, different agent).
  The artifacts ARE the orchestrator's memory.

## When this discipline is overhead

- For trivial, single-step work where the cost of a wrong turn
  is small.
- For exploratory work where the right answer is unknown and
  we're feeling our way.
- For interactive work where the human is in the loop at every
  step.

The discipline scales with the cost of failure. Don't pay for
ceremony you don't need. But for arcs that touch the substrate,
or that ship a check, or that establish a convention — failure
engineering is the path.

## What this realization adds to the substrate

This document IS the realization. It names the discipline that
was already informally being practiced. Future arcs that face
agent-dispatch decisions reference this REALIZATIONS doc + the
artifacts it cites (BRIEF, EXPECTATIONS, SCORE, REALIZATIONS) as
the canonical playbook.

The next time an arc needs an agent sweep, the orchestrator
knows: write the brief + expectations + commit BEFORE spawning;
score against fixed criteria; treat failures as data; surface
upstream defects as new arcs; reland with amendments.

## Cross-references

- `BRIEF-SLICE-1.md` — original brief; preserved as
  first-attempt record.
- `EXPECTATIONS-SLICE-1.md` — original scorecard.
- `SCORE-SLICE-1.md` — first attempt's actual score; the
  document that triggered arc 128.
- `BRIEF-SLICE-1-RELAND.md` — amendments + reland brief.
- `EXPECTATIONS-SLICE-1-RELAND.md` — 8-row reland scorecard.
- `SCORE-SLICE-1-RELAND.md` — second attempt's clean 14-of-14
  scoring; the document that validates artifacts-as-teaching.
- `BRIEF-SLICE-2.md` — slice 2 handoff to convert
  `:ignore` → `:should-panic`. Tests whether the substring
  propagates through `run-sandboxed-hermetic-ast` →
  `TestResult::Failure` → cargo libtest panic.
- `../128-check-walker-sandbox-boundary/INSCRIPTION.md` — the
  substrate fix that arc 126's failure surfaced.
- `../125-rpc-deadlock-prevention/DESIGN.md` — the WITHDRAWN
  type-precise rule; the four questions killed it.
- `../127-thread-process-symmetry/DESIGN.md` — the WITHDRAWN
  architectural rethink; the four questions + ZERO-MUTEX.md
  killed it.
- Memory: `feedback_four_questions.md` — the doctrine that
  guards every artifact.
- Memory: `feedback_proposal_process.md` — rejected proposals
  stay.
- `docs/SUBSTRATE-AS-TEACHER.md` — the substrate-level
  precedent that failure engineering generalizes.
