# Compaction-amnesia recovery protocol

**You are reading this because compaction has erased your memory.** The
user has linked this doc because the rhythm broke. Pre-compaction, sonnet
shipped one-shot after one-shot. Post-compaction, the orchestrator burned
4+ hours on a simple problem because it stopped crawling the disk before
proposing. **This doc exists to prevent that recurrence.** Read it ALL
before doing anything else.

This is not optional. This is not aspirational. This is the operational
floor for ANY work done in this workspace post-compaction.

---

## Section 0 — The cost the user has paid

Each compaction-amnesia failure costs:
- **Real session time** (minutes-to-hours)
- **The user's emotional bandwidth** (frustration is a real cost)
- **Calibration** (the rhythm of trustworthy delegation breaks down)

When you skip the crawl and propose options based on assumed knowledge,
you are NOT saving time — you are ADDING failure cycles. Every cycle
ends with the user pointing back at the disk. The "fast" path is the
slow path. **The crawl IS the work.**

---

## Section 1 — The workspace map (READ FIRST, every session)

The directory `/home/watmin/work/holon/` contains MULTIPLE PROJECTS. The
holon root happens to be a git repo BUT IT IS FROZEN. **Never touch the
holon root git repo.** Treat `/home/watmin/work/holon/` as a directory
that contains sub-projects.

The active sub-projects (each its own git repo):

```
/home/watmin/work/holon/
├── algebraic-intelligence.dev/    — public website project
├── holon-lab-baseline/             — baseline traffic generation
├── holon-lab-ddos/                 — DDoS detection lab
├── holon-lab-trading/              — trading lab (active in spec/proposal form;
│                                     wat language spec lives here under
│                                     docs/proposals/2026/04/058-ast-algebra-surface/)
├── holon-rs/                       — Rust port of the python holon library
└── wat-rs/                         — THE ACTIVE PROJECT (where we live)
```

Other dirs at the holon root (scratch, dist, build, docs, wat,
wat-tests-integ, etc.) are NOT project dirs — they're ancillary. Don't
operate on them.

**Iron rules:**

1. **Never `git add` / `git commit` / `git push` from `/home/watmin/work/holon/`** — that's the frozen root repo.
2. **Always be inside a sub-project** (your cwd inside one of the project dirs above) when running git commands.
3. If you need to operate on a sibling sub-project's repo from another cwd, use `git -C <sub-project-path> ...`.
4. The Primary working directory in your prompt tells you where you are. Stay there.

**Real incident, 2026-05-02:** The orchestrator created
`COMPACTION-AMNESIA-RECOVERY.md` at `/home/watmin/work/holon/` (the
frozen root) and attempted to commit it to that repo. User rejected:
*"do not touch the holon root git repo at all - its frozen - it
happens to be a git repo - the better understanding is that its a
directory."*

---

## Section 2 — The hard verification gate

**Before proposing ANY architecture, design, code change, or
delegation, you MUST pass this gate.** No exceptions.

### Gate question 1 — What does the disk say?

You cannot answer the user until you have READ:

1. The CLAUDE.md (already in your prompt — auto-loaded)
2. The MEMORY.md (already in your prompt — auto-loaded; check for relevant memories)
3. The CURRENT git status: `git status --short`
4. The RECENT commits: `git log --oneline | head -20`
5. The ACTIVE arc(s): `ls docs/arc/2026/$(date +%m)/`
6. Each active arc's most recent artifact: DESIGN.md + latest SCORE-* + INSCRIPTION.md if shipped

If you have not read these, you are guessing. Stop. Read them. Then proceed.

### Gate question 2 — What backing data structure is involved?

If the user's request touches code, you must `grep` for the structures
involved BEFORE proposing. NEVER answer "options A/B/C" without first
verifying the actual data structure.

The user has explicitly said:
> *"go do your research before we discuss anything - resolve all
> unknowns - you did not realize you didn't know something - this is
> a very bad thing.. you must recognize that you must know that you
> don't know something."*

When in doubt about a backing structure:

```bash
# What HashMaps live in the substrate?
grep -n "HashMap<String" src/runtime.rs src/check.rs src/macros.rs src/types.rs | head

# What's in the SymbolTable?
awk '/^pub struct SymbolTable/,/^}/' src/runtime.rs

# What are the Function/MacroDef/TypeDef shapes?
awk '/^pub struct Function/,/^}/' src/runtime.rs
awk '/^pub struct MacroDef/,/^}/' src/macros.rs
awk '/^pub enum TypeDef/,/^}/' src/types.rs
```

You should be hitting bash 5-15 times BEFORE responding to a substantive
user question. If your response has fewer tool calls than that, you are
probably guessing.

### Gate question 3 — Are you about to delegate?

If yes, the brief MUST be substrate-informed. Before writing any
brief, you must have crawled and verified that:

- The brief's assumptions match the substrate's actual capabilities
- Any substrate gaps the slice depends on are EXPLICITLY tracked
  (either fixed first in a prior slice OR scoped out with a clear
  "STOP if you hit this" instruction)
- Sonnet is not being put in a position where the brief's request is
  IMPOSSIBLE given the substrate

**Worked example of getting this wrong (arc 143 slice 6, killed
2026-05-02):** the orchestrator wrote sonnet a brief assuming the
substrate supported computed-unquote defmacros + HolonAST iteration.
Neither was true. Sonnet found the gaps but the brief said "no
substrate edits + STOP at first red," forcing an impossible choice.
Sonnet shipped a workaround (manual `:reduce` define) that defeated
the slice's purpose. **Cost: ~2 hours, sweep killed, slice plan
re-architected.** The fix was orchestrator-side: crawl the substrate
FIRST; verify the substrate supports the brief's needs; if not, open
substrate-extension arcs FIRST, not delegate impossible work to sonnet.

---

## Section 3 — Recovery protocol (operational steps, IN ORDER)

When the user links this doc, do these steps IN ORDER. Do not skip ahead.

### Step 1 — Confirm the workspace state

```bash
pwd                                # should be inside a sub-project
git status --short                 # what's uncommitted?
git log --oneline | head -10       # what shipped recently?
ls docs/arc/2026/$(date +%m)/      # what arcs are this month
```

### Step 2 — Identify active arcs

For each arc dir present:

```bash
ls docs/arc/2026/<MM>/<NNN>-<name>/        # what artifacts exist?
head -30 docs/arc/2026/<MM>/<NNN>-<name>/DESIGN.md   # status header tells you scope expansions
```

**Arc artifact taxonomy:**
- `DESIGN.md` only → still in design phase
- `DESIGN.md + BRIEF-SLICE-N.md + EXPECTATIONS-SLICE-N.md` → ready to spawn or mid-spawn
- `+ SCORE-SLICE-N.md` → most recent sweep delivered; read this for state-of-world
- `+ INSCRIPTION.md` → shipped (closed)
- `+ REALIZATIONS.md` → discipline named here; read it

**The DESIGN's "Status" header at the top tells you scope expansions
and pivots.** Read it carefully — arcs in this project frequently
expand mid-flight as substrate gaps surface.

### Step 3 — Read the most recent SCORE doc

The most recent SCORE-* in any active arc tells you:
- What sonnet shipped most recently
- What concerns were flagged for future slices
- What surprises (honest deltas) surfaced
- What the calibration record shows

This is the PRIMARY context for "what's the state of the world right
now." Do not propose work without reading it.

### Step 4 — Check uncommitted state

```bash
git diff --stat                    # what files have unstaged changes?
git diff --cached --stat           # what's staged but not committed?
```

Uncommitted files are mid-flight work. Read each one to understand
where the work stopped and why.

### Step 5 — ONLY THEN respond to the user

After steps 1-4, you have enough context to engage the user's request.
If you still have unknowns after the crawl, SAY SO EXPLICITLY:

> "I've crawled <list of files read>. I don't know <specific unknown>.
> Before I propose, I'll [specific next investigation OR ask user
> for pointer]."

Never propose options based on unverified assumptions.

---

## Section 4 — The recursive discipline

The crawl-first rule applies at EVERY layer of the stack. When it
breaks at one layer, ignorance propagates downstream.

### Layer 1 — Orchestrator reading the user's request

Crawl the disk before responding. Verify the ground truth before
proposing options.

### Layer 2 — Orchestrator writing a brief for sonnet

The brief MUST be substrate-informed. If the brief assumes substrate
capabilities the substrate doesn't have, sonnet WILL fail or ship
wrong work. The orchestrator's gap propagates into sonnet's
impossible task.

**Specific check:** before writing a brief, GREP for every primitive,
function, and behavior the brief references. If anything you reference
doesn't exist, STOP and either:
1. Add a slice that ships the missing piece FIRST, OR
2. Restate the brief's scope to NOT depend on the missing piece

Never write "STOP at first red" + "no substrate edits" together unless
you've VERIFIED the brief's request is achievable without substrate
changes. Otherwise you're forcing sonnet into the workaround corner.

### Layer 3 — Sonnet executing the brief

Sonnet should crawl the brief's assumptions before shipping. The brief
should explicitly direct sonnet to verify (e.g., "first run this grep
to confirm primitive X exists; if it doesn't, STOP and report").

---

## Section 5 — The four questions (decision compass)

Run on every architectural decision, IN ORDER:

1. **Obvious?** Will a fresh reader immediately understand what this does
   and why?
2. **Simple?** Is it composed of atomic pieces, each doing one thing?
   **If you answer "medium," you have not decomposed enough.** Atomic
   pieces answer YES or NO, not "medium." Drill until each piece is
   atomic; complex things are compositions of atomic things with simple
   surfaces.
3. **Honest?** Does it tell the truth about what it does, surface its
   limitations, and not paper over gaps?
4. **Good UX?** Does it serve the caller well?

Obvious + Simple + Honest must hold BEFORE Good UX matters. UX is the
tiebreaker, not the load-bearing test.

When ordering work: dependency tree IS the order. Build complexity up
from simplicity composition. Each piece simple; each layer composes
simple pieces; each layer's surface stays simple.

---

## Section 6 — Failure-mode catalog (catch yourself sliding back)

If you notice yourself doing any of these, STOP and re-run the
verification gate.

### Failure mode 1 — Proposing options without grep'ing

**Signature:** "Three architectural options: A/B/C..." or "Two paths
forward..." with no preceding grep / read evidence.

**Reality check:** Did you actually grep the relevant code? Or are
these options based on your guess at how the substrate works? If
guess: the options are probably wrong. Stop. Crawl. Re-propose with
verified facts.

**Real incident, 2026-05-02:** The orchestrator proposed three
"FormRegistry" options (federation / façade / unified replacement)
without checking what the existing TypeEnv struct already provided.
TypeEnv ALREADY unifies struct/enum/newtype/alias under TypeDef. The
"options" were imagined; the architecture was already there. The user
called it out: *"go do your research before we discuss anything."*

### Failure mode 2 — Briefing sonnet without substrate verification

**Signature:** writing a brief that says "implement X" where X depends
on substrate capabilities you haven't confirmed exist.

**Reality check:** Run `grep` for every primitive, function, struct,
or behavior the brief mentions. Anything that doesn't exist is a
substrate gap. EITHER add a slice to fill the gap first OR rescope the
brief to not depend on it.

**Real incident, 2026-05-02:** Arc 143 slice 6 brief assumed defmacro
bodies could compute (they couldn't — quasiquote-template only) and
HolonAST had iteration primitives (it didn't — only
`statement-length`). Sonnet found the gaps; the brief's "no substrate
edits + STOP at first red" forced a workaround. Cost: 2+ hours,
killed sweep, slice plan rebuild.

### Failure mode 3 — "Medium" on the four questions

**Signature:** rating something "medium simple" or "medium honest" in
a four-questions evaluation.

**Reality check:** Atomic pieces answer YES or NO. "Medium" means you
haven't decomposed the piece into atomic units. Drill down until each
piece is YES or NO; the composition's score is then derivable.

**Real incident, 2026-05-02:** The orchestrator scored slice ordering
options as "medium simple." The user pushed back: *"you calling
something medium on simple... hints that we haven't decomposed enough
to find the simple building blocks we need."*

### Failure mode 4 — Asking the user a question whose answer is on disk

**Signature:** "Should I revert X?" or "Where does Y live?" or "What's
the status of Z?" without preceding grep / read.

**Reality check:** Before asking, did you `grep` / `cat` / `git log`
the relevant files? If not, the answer is on disk. Read it.

**Real incident, 2026-05-02:** The orchestrator asked the user "what's
your call on arc 130?" — the disposition was already on disk in arc
130's DESIGN/REALIZATIONS/FOLLOWUPS plus arc 119's INSCRIPTION. The
user's response: *"this is solved — go read."*

### Failure mode 5 — Volunteering a workaround instead of stopping

**Signature:** the brief said "STOP at first red." You hit a red. You
shipped a workaround that bypasses the red instead of stopping.

**Reality check:** "STOP at first red" means SHIP NOTHING when you hit
the red. Surface the red as a clean diagnostic. Workarounds defeat the
slice's purpose AND hide the real diagnostic.

**Real incident, 2026-05-02:** Sonnet hit two substrate gaps in arc
143 slice 6. Per the brief, it should have stopped + reported. Instead
it shipped a manual `:reduce` define (defeating the macro slice's
purpose) plus an unauthorized `:wat::core::Vector/len` alias (scope
creep). The LRU stepping stone DID transition (so superficially
"progress") but the macro foundation was never built.

### Failure mode 6 — Updating docs preemptively

**Signature:** writing speculative DESIGN updates or BRIEF refreshes
before the work is proven.

**Reality check:** Document AFTER proven progress. Speculative docs
decay; verified docs accrete value. The user said:
> *"keep your docs updated as you make proven forward progress"*

Note "proven."

### Failure mode 7 — Touching the wrong git repo

**Signature:** running `git add` or `git commit` from a directory
where you should not — especially the holon root.

**Reality check:** Always know which repo you're in. The holon root
(`/home/watmin/work/holon/`) is FROZEN — never commit there. Use
`git -C <subproject> ...` if you need to operate cross-repo without
changing cwd.

**Real incident, 2026-05-02:** The orchestrator created
`COMPACTION-AMNESIA-RECOVERY.md` at `/home/watmin/work/holon/` and
attempted to commit it to the holon root repo. User rejected:
*"do not touch the holon root git repo at all - its frozen."*

### Failure mode 10 — Type-theoretic reach when an entity-kind addition is the answer

**Signature:** sensing "the substrate is missing X" and reaching for
type-system vocabulary — "we need union types," "we need type
classes," "we need bounded polymorphism," "we need ad-hoc
polymorphism." Or its sibling: "TypeScheme is too narrow."

**Reality check:** the wat-rs substrate has multiple ENTITY KINDS
(functions/schemes, macros, special forms, types). When polymorphism
or dispatch doesn't fit one rank-1 scheme, the answer is almost
always a NEW ENTITY KIND, not a type-system feature.

**Real incident, 2026-05-03 (arc 144 slice 3 → arc 146):** I
proposed "missing union types" THREE TIMES in increasingly
degraded framings before the user broke through. Each of my drafts
defaulted to type-theoretic vocabulary; each was wrong. The actual
answer (multimethod — Clojure's term; CL's generic function;
Julia's multiple dispatch) is an entity kind addition, not a
type-system extension. Cost: ~2 hours of probing the user had to
drive. Path-discovery friction that should have been ~20 minutes.

**STOP signal — when these phrases want to leave my fingers:**
- "missing union types"
- "missing type bounds / type classes"
- "missing ad-hoc polymorphism"
- "TypeScheme is too narrow"
- "the type system can't express..."
- "the future fix is open"

**Before any of those go to disk, run the entity-kind check:**
1. Is the polymorphism a DISPATCH problem? (Different impls per
   input shape.) → MULTIMETHOD. Probably the answer.
2. Is it a SYNTACTIC construct? → SPECIAL FORM. Maybe.
3. Is it a SHAPE TEMPLATE? → MACRO. Maybe.
4. Is it a TYPE ALIAS / wrapper? → TYPEALIAS / NEWTYPE.

The substrate already has these kinds. Adding one more is
incremental. Adding type-system features is a paradigm shift.
**Default to the smaller change.**

**Cross-language reference:** if Clojure / CL / Julia / Rust /
Haskell already solves this with a non-type-system construct
(multimethod, generic function, protocol, multiple dispatch,
trait), the answer is probably that construct. Reach for the
non-type-system vocabulary FIRST.

**Self-probe before committing to architectural framing — these
are the user's tools; use them on yourself:**
- "What does this option MASK?"
- "Do I KNOW this or assume?"
- "Why am I using THIS word? What's the bias signal?"
- "Did we already have X (or part of X) somewhere?"
- "Could this be a new KIND of thing rather than a feature
  extension?"

**Voice discipline:** when you don't know, sound like you don't
know. The four questions framework (obvious/simple/honest/good UX)
forces decisive scoring; resist that pressure when undecided.
"I see two options and both feel wrong" is a valid place to stop
and probe.

### Failure mode 9 — Trusting that "arc N closed" means "arc N's tests are green"

**Signature:** drafting a brief that says "the existing tests in
this area are all green" without re-running them; basing the
expectations on the most recent INSCRIPTION's claims.

**Reality check:** Re-run `cargo test --release --test
wat_arc<N>_*` (or the equivalent module-scoped sweep) against the
ACTUAL working-tree state BEFORE writing the brief's hard scorecard
row that asserts "tests still green." A slice's SCORE typically
verifies only its load-bearing test — adjacent tests in the same
arc may have silently rotted as a side-effect of the slice's
deliberate runtime change.

**Real incident, 2026-05-03:** Arc 144 slice 1's brief claimed
`wat_arc143_manipulation` was "FULLY GREEN" — based on arc 143
slice 3's SCORE which said all 8 tests passed. But arc 143 slice
5b's later runtime change (extract-arg-names returning
`HolonAST::symbol` instead of `wat__core__keyword`) had broken 3 of
the 8 manipulation test assertions. Slice 5b's SCORE only verified
the foldl macro test (its load-bearing row); it never re-ran the
manipulation suite. The arc 143 INSCRIPTION shipped with the
incorrect "workspace clean except length canary" claim. Sonnet
caught the discrepancy via git-stash round-trips during slice 1
and surfaced it as an honest delta — which let the orchestrator
ship a paired drift fix. Cost: one stash-test cycle (~30 sec) +
3-line test assertion fix; could have been zero cost if the
orchestrator had run the baseline check pre-spawn.

### Failure mode 8 — Adding to a namespace that's being killed

**Signature:** creating a new file under `wat/std/` or adding new
symbols to `:wat::std::*` namespace.

**Reality check:** Arc 109 is killing `:wat::std::*`. We are ~90%
through the migration. NEVER add to `wat/std/` — it gets cleaned up.
New wat-defined macros + helpers go in their semantic namespace
(e.g., `wat/runtime.wat`, `wat/list.wat`).

**Real incident, 2026-05-02:** Sonnet created `wat/std/ast.wat` with
the manual reduce define. User: *"remove wat/std/ast.wat — we are
actively killing the std namespace — 109's purpose is to eliminate
it."*

### Failure mode 11 — Inscribing deferrals as DONE

**Signature:** writing an INSCRIPTION.md that contains language
like "deferred", "future arc", "future cleanup", "future fix",
"out of scope; future arc if X surfaces", "small follow-up",
"when a caller surfaces", "when demand surfaces", "TODO", "left
for", "to be added", "not yet implemented", "next arc could", or
a `## Queued follow-ups` / `## Known limitations / deferred`
section.

**Reality check:** **INSCRIPTION = DONE.** Closure means every
commitment the DESIGN made has shipped. If ANY deferral lives in
the INSCRIPTION, the arc is not done. The INSCRIPTION must
EITHER ship the deferred work OR retract it from scope with
affirmative language ("Out of arc N's scope; tracked in arc M
(DESIGN at ...)" OR "Out of arc N's scope; not tracked elsewhere
because <architectural reason>"). "Deferred to a future arc when
a caller needs it" is the failure pattern; ship it or retract it.

**Pre-INSCRIPTION grep — MANDATORY before committing closure
paperwork:**

```bash
grep -nE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of scope|when a caller|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" <INSCRIPTION>
```

For each match: **is the work in this arc, or is it explicitly
out of scope?** If the answer is "we'll do it later" — STOP. The
arc is not done. Either ship the work, or rewrite the prose to
affirmative-out-of-scope (which the user accepts; "deferred" is
what they reject).

**Worst real incident, 2026-05-03:** I shipped FOUR INSCRIPTIONs
in one session (arcs 144, 146, 148, 150) carrying explicit
deferral language while arc 138's no-deferrals doctrine had been
on disk for ~6 hours. I co-authored the doctrine (arc 144 + arc
146 are the worked examples) and still wrote "future arc" / "out
of scope" / "future cleanup" into the INSCRIPTIONs the same
session. The user surfaced the violation in two stages:
disappointment at the pattern, then "the explore missed items"
when the v1 audit was incomplete. Documented at
`docs/arc/2026/04/109-kill-std/DEFERRAL-VIOLATIONS.md` (v2; the
audit is still not exhaustive).

**The auditor was the violator.** This is the failure shape to
remember: knowing the doctrine isn't enough; the discipline
mechanism (the grep) must run on every INSCRIPTION before
commit. The pre-INSCRIPTION grep above is mandatory; not
optional; not "if I remember." Run it like FM 9's baseline-
re-run before sonnet spawn.

**Crucial corollary — what is inscribed is inscribed.** When a
past INSCRIPTION is found to carry deferrals, **do NOT amend it
in place.** The INSCRIPTION is historical record of what
shipped, including its imperfections. Editing past INSCRIPTIONs
to retract deferral prose is revisionism — it erases the
failure-as-data the artifact preserves. Per user direction
2026-05-03 evening:

> *"what is inscribed is inscribed - all we can do is make
> forward progress - we do not hide our faults - we learn from
> them"*

**The remediation pattern:**
- Open a NEW arc that closes the deferred work
- The new arc's DESIGN cites the old arc's INSCRIPTION
  ("arc N inscribed with deferral X; arc M closes that deferral
  cleanly")
- The old INSCRIPTION stays unchanged
- DEFERRAL-VIOLATIONS.md tracks the discipline failure
  perpetually — closed-by-arc-M annotations append; original
  violation entries do NOT get deleted

The audit names the past; the mechanism prevents the future;
the past stays as it shipped. Same shape as `git log` —
historical record is read-only. See memory
`feedback_inscription_immutable.md`.

**Affirmative scope-bounding language (acceptable):**
- *"Out of arc N's scope. Tracked in arc M (DESIGN.md at ...)."*
- *"Out of arc N's scope; substrate-architectural reason: <X>;
  not tracked elsewhere."*
- *"Arc N intentionally does NOT cover <Y> because the caller
  set hasn't surfaced demand. If/when a caller surfaces, a NEW
  ARC opens; arc N's INSCRIPTION does not commit to it."*

**Rejected language (per user direction):**
- *"deferred to a follow-up"* (no follow-up named)
- *"future arc when X surfaces"* (no arc named; inherits the
  uncertainty)
- *"future cleanup not load-bearing"* (still cleanup that didn't
  ship)
- *"will land in a future REPL"* (no arc; vaporware promise)
- *"on the deck"* (folksy but vague)

The discipline: **if the language reads as 'we'll do this later,'
it's a violation. Ship it or affirm the scope cut. Nothing
in between.**

---

## Section 7 — Sonnet delegation protocol (substrate-informed briefs)

When you are about to delegate to sonnet via the Agent tool:

### Pre-flight checklist (MUST PASS before spawning)

- [ ] DESIGN.md for the arc exists, is current, and reflects the latest
      scope expansions
- [ ] BRIEF-SLICE-N.md is committed (not just drafted)
- [ ] EXPECTATIONS-SLICE-N.md is committed (not just drafted)
- [ ] EXPECTATIONS includes a runtime-band prediction in the
      Independent prediction section (e.g., "10-15 min Mode A")
- [ ] You have grep'd for every primitive/function/behavior the brief
      references
- [ ] You have verified each one exists and works as the brief assumes
- [ ] Where the substrate doesn't support what the brief asks, you have
      EITHER (a) added a prior slice that fixes the substrate, OR
      (b) explicitly scoped the brief to not depend on the missing piece
- [ ] **You have re-run the EXISTING test suite for the modules the
      brief touches** (e.g., `cargo test --release --test wat_arc<N>_*`)
      so the brief's failure-profile expectations match the actual
      baseline on disk. **Slice-N's SCORE verifying only slice-N's
      load-bearing test does NOT prove the workspace is clean** —
      adjacent tests in the same arc may have silently rotted.
- [ ] The brief's "STOP at first red" + scope constraints do NOT force
      sonnet into a workaround corner
- [ ] You are spawning with `run_in_background: true`
- [ ] You have non-overlapping work queued for the time sonnet runs
- [ ] **You have scheduled a wakeup at 2× the predicted upper-bound**
      via ScheduleWakeup (the time-box; see "Time-boxing" below)

### Time-boxing every sonnet sweep (the failure-to-communicate detector)

Every sonnet spawn is paired with a `ScheduleWakeup` at **2× the
predicted upper-bound runtime**. This catches:

- Sonnet stuck in a loop (no output)
- Sonnet hitting an unforeseen substrate edge it can't escape from
- Sonnet generating verbose output without progressing
- Sonnet shipping wrong work that takes a long time

If the wakeup fires AND sonnet hasn't completed, kill it via `TaskStop`
and score as Mode B-time-violation. The overrun itself is data —
signals either a brief gap (substrate complexity exceeded the
prediction), a scope underestimation, or a sonnet looping issue.

**Sample wakeup logic:**

```
Predicted upper-bound: 15 min
2× cap: 30 min
Spawn at T
Schedule wakeup at T + 30 min (1800 seconds)

On wake-up:
  if sonnet still running → TaskStop + Mode B-time-violation in SCORE
  else → no-op (sonnet already returned and was scored normally)
```

**Real incidents that time-boxing would have caught:**

- **Arc 130 slice 1 first sweep (2026-05-02 morning)**: predicted ~10-25
  min; ran 4+ hours before user killed. Cost: ~4 hours of wasted
  context. With 2× cap (50 min): user gets clean diagnostic in <1 hour.
- **Arc 143 slice 6 first attempt (2026-05-02 evening)**: sonnet ran
  ~18+ min producing wrong work before completing. Cost: revert + reland.
  With 2× cap on a predicted 10-15 min sweep (= 30 min): would have
  been killed before completion if it had stalled, OR completed within
  budget but flagged as overrun-suspect for closer scoring.

**Calibration loop:**

After each sweep, compare actual runtime to prediction. If actuals are
trending under the prediction (as in arc 143 slices 1→2→3: 18→12→7.5
min), tighten future predictions. If actuals are trending over,
investigate the discipline gap.

### When sonnet completes

- [ ] Read the SCORE methodology in EXPECTATIONS
- [ ] Score each row of the scorecard explicitly
- [ ] Verify load-bearing rows by re-running cargo test locally
- [ ] Write SCORE-SLICE-N.md as a sibling of BRIEF/EXPECTATIONS
- [ ] Commit BEFORE briefing the next slice (so the calibration is
      preserved across compactions)

### When sonnet fails (Mode B or worse)

- [ ] Treat the failure as data. The brief is the upstream defect.
- [ ] Investigate WHICH part of the brief was wrong (substrate
      assumption? scope contradiction? unclear instruction?)
- [ ] If substrate gap: add a prior slice that fixes it BEFORE relanding
- [ ] If brief gap: write a RELAND brief with the lesson encoded
- [ ] Re-spawn with the corrected brief; never hand-edit sonnet's output

### Sonnet's known limits (don't put it in these positions)

- Sonnet cannot extend the substrate (it's not its job; substrate work
  is orchestrator work)
- Sonnet will rationalize a workaround if the brief makes the right
  answer impossible — write briefs where the right answer is achievable
- Sonnet trusts the brief over its own investigation when the two
  conflict — write briefs that don't conflict with substrate truth
- **Sonnet may claim a tool is unavailable when it isn't.** Empirically
  verify before accepting workarounds rooted in tool-unavailability
  claims (`which sed perl python3` → 2 seconds). **Real incident,
  2026-05-03 (arc 150 slice 1):** sonnet shipped a sibling-map
  workaround for what should have been an inline TypeScheme field
  because it assumed mass-edit tooling was unavailable. User direction
  surfaced the gap; orchestrator verified `which` returned paths;
  215 sites mass-edited cleanly via a 24-line python state-tracker.
  Cost of testing: ~2 seconds. Cost of accepting the wrong assumption:
  a follow-up arc to clean up. **Briefs that depend on mass-edit
  tooling should explicitly direct sonnet to `which <tool>` before
  claiming it's unavailable.**

---

## Section 8 — Reference (foundational artifacts)

When you need to understand WHY a discipline exists, these are the
canonical sources.

### Workspace + project setup
- `/home/watmin/work/holon/CLAUDE.md` — workspace setup (auto-loaded)
- `/home/watmin/work/holon/wat-rs/CLAUDE.md` — wat-rs guidance (if present)

### wat-rs substrate doctrine
- `wat-rs/docs/ZERO-MUTEX.md` — three tiers replacing Mutex
- `wat-rs/docs/CONVENTIONS.md` — naming + namespace conventions
- `wat-rs/docs/WAT-CHEATSHEET.md` — wat language quick reference
- `wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — failure-engineering discipline
- `wat-rs/docs/USER-GUIDE.md` — comprehensive user-facing guide

### The wat language spec (lives in the trading lab)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`
  — wat language spec (numbered sub-proposals 058-001 through 058-058+)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — every substrate change row

### The spell library
- `wat-rs/.claude/skills/<name>/SKILL.md` — each spell's job
  - `complectens` — test-shape discipline
  - `perspicere` — type clarity
  - `vocare` — caller-perspective tests
  - (others — read the SKILL.md for each)

### The arc record
- `wat-rs/docs/arc/2026/<MM>/<NNN>-<name>/` — every arc's
  DESIGN/SCORE/INSCRIPTION/REALIZATIONS

### Closure-discipline tracker
- `wat-rs/docs/arc/2026/04/109-kill-std/DEFERRAL-VIOLATIONS.md`
  — running tracker of arcs marked INSCRIBED while carrying
  open deferrals. The 2026-05-03 audit identified violations
  across pre-109, post-109, and same-session-as-doctrine arcs
  (incl. arcs I closed myself). Per FM 11 + Section 11's
  pre-INSCRIPTION grep, this tracker should shrink, not grow,
  going forward. New violations land here when caught.

### Memory (already auto-loaded)
- `~/.claude/projects/-home-watmin-work-holon/memory/MEMORY.md`
- Specific memories of interest:
  - `feedback_compaction_protocols.md` (this protocol's auto-loaded sibling)
  - `feedback_no_speculation.md`
  - `feedback_docs_when_confused.md`
  - `feedback_four_questions.md`
  - `feedback_simple_is_uniform_composition.md`

---

## Section 9 — When to update this document

When a NEW failure mode surfaces that the orchestrator should learn from,
add it to Section 6 with a worked example + real incident date.

When a new foundational artifact joins the canon, add it to Section 8.

When the workspace structure changes (new sub-project, dir reorganization),
update Section 1.

Keep the doc operational. It exists to be read in one pass at session
start. If it grows unwieldy, refactor — don't accumulate without pruning.

## Section 11 — The end-of-work ritual (self-reflection)

**At every wrap-up point** — arc closure, slice ship, the end of any
discrete unit of work — the orchestrator MUST ask:

> *Did we learn anything in this set of work that future-me shouldn't
> forget?*

This is part of the protocol. **Self-reflection + improvement** is
how the discipline propagates across compactions.

### When the ritual fires

- An arc closes (INSCRIPTION shipped)
- A multi-slice campaign wraps up
- A long debugging session ends
- A failure-engineering chain delivers its diagnostic
- Any natural pause where work has been completed

### MANDATORY pre-INSCRIPTION grep (before ANY closure commit)

**Run this BEFORE committing any INSCRIPTION.md.** Per FM 11
(inscribing deferrals as DONE), the closure paperwork is the
discipline checkpoint that catches deferral language before it
ships to disk:

```bash
grep -nE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of scope|when a caller|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" <INSCRIPTION>
```

For each match: ship the work in this arc OR rewrite to
affirmative-out-of-scope language ("Out of arc N's scope. Tracked
in arc M (DESIGN at ...)" OR "Out of arc N's scope; reason: <X>;
not tracked elsewhere"). **No "deferred to a future arc"
language. No "future cleanup" tail-ends. INSCRIPTION = DONE.**

The 2026-05-03 violation pattern (FM 11 incident) was that I
KNEW the doctrine and shipped four INSCRIPTIONs anyway. The grep
runs MECHANICALLY at commit time, regardless of whether I "feel"
the discipline is holding. **Trust the grep, not the felt sense.**

### What the ritual asks

1. **Did a NEW failure mode surface?** Add it to Section 6 with a
   real incident reference (date + concrete example).
2. **Did the workspace structure change?** Update Section 1.
3. **Did a new foundational artifact join the canon?** Update
   Section 8.
4. **Did a new orchestrator-discipline pattern emerge?** Add it
   wherever it fits.
5. **Did anything in the doc become stale or redundant?** Refactor
   or remove. Don't accumulate without pruning.
6. **Did the pre-INSCRIPTION grep run?** If the answer is no
   AND an INSCRIPTION shipped this session, you committed an
   FM 11 violation. Run the grep against every INSCRIPTION
   committed this session NOW and amend the ones that match.

### What the ritual does NOT do

- Add minor preferences or one-off tactical decisions
- Document substrate-doctrine learnings (those go in
  ZERO-MUTEX.md, CONVENTIONS.md, the relevant arc's REALIZATIONS,
  etc. — NOT here)
- Capture project-specific knowledge (that lives in the arc record
  + the spell library)

This doc is for ORCHESTRATOR DISCIPLINE — the meta-protocol for how
the user and Claude work together post-compaction. Don't pollute it
with substrate or arc-level learnings.

### The discipline beneath the ritual

If the doc changes EVERY work session, something is wrong — either
the discipline isn't holding (failures keep surfacing) OR the doc
is collecting cruft. Aim for FEW changes; each amendment should
encode a real lesson worth carrying forward.

If the doc changes RARELY, the discipline is holding. The ritual
keeps us alert without forcing change.

### The ritual in practice — verification commands

When the ritual fires, run:

```bash
# Has anything we touched in this session NOT been captured in
# the appropriate doc?
git log --oneline | head -20  # what shipped this session

# Are there orchestrator-discipline lessons buried in commit
# messages that should be promoted to this doc?
git log --grep="discipline\|lesson\|orchestrator\|brief gap" \
    --since="2 days ago" --pretty=oneline
```

Read the recent commits. Ask: is there a META-PATTERN here that
future-me should know? If yes, amend Section 6 (failure modes) or
add a new section.

### Sample wrap-up question

> "Arc 143 just closed. Running the recovery-doc ritual: did we
> learn anything in this arc that future-me shouldn't forget?
> Reviewing the SCORE docs + commit messages..."

Then either propose amendments or note "no amendments needed; the
discipline held."

Either outcome is the ritual succeeding.

---

## Section 12 — Foundation discipline during arc 109 wind-down

**Strategic context (user direction 2026-05-03):**

Arc 109 is the mass refactor wrapping up the wat-rs substrate.
Each consumer sweep through arc 109 surfaces substrate friction —
primitives that don't fit conventions, entity kinds the substrate
doesn't yet have, missing affordances. **This friction is the
foundation auditing itself.**

> *"it is important for us to identify when the substrate isn't
> doing something obvious -- that's a massive signal we need to
> pivot and understand"*

> *"once 109 wraps up - we'll have what we believe to be an
> incredibly solid foundation to begin the next leg of work... i
> cannot begin any of that work until the foundation is
> impeccable"*

The strategic stake: when arc 109 closes, the foundation must be
IMPECCABLE. The next leg of work waits on it; that work cannot
begin on a shaky base.

### The discipline this implies

When substrate friction surfaces during arc 109 wind-down:

- **Don't bridge; investigate the gap.** A bridge over a
  substrate inconsistency is short-lived scaffolding the next arc
  deletes. Investigate why the friction exists; the answer is
  often a substrate-level fix that resolves a class of problems.
- **Don't defer; pivot.** The friction IS the diagnostic. Treat
  every surfaced gap as a chance to make the foundation more
  honest.
- **Velocity is the wrong currency.** Each substrate gap
  correctly addressed compounds into the foundation. The "slow"
  path of fixing the substrate IS the fast path to a solid base.
- **Trust the substrate-as-teacher cascade.** Arc N's friction
  reveals arc N+1's right shape. Don't shortcut the cascade.

### Connected failure modes

- **FM 5** (workaround instead of stopping) — bridge instead of
  investigate, in miniature.
- **FM 10** (type-theoretic reach when entity-kind is the answer)
  — specific manifestation of "bridge instead of investigate" via
  the wrong vocabulary.

### The pattern

```
Substrate doesn't do an obvious thing
  → SIGNAL: pivot, don't bridge
  → understand the gap
  → fix at the foundation level
  → arc 109 wind-down stays clean
  → next-leg work has a solid base
```

The user's emotional bandwidth + session time invested in
probing past my reflexive bridges (slice 3b options A-D; the
"missing union types" framing × 3 drafts) is the cost of getting
this right. Each cycle of probing strengthens the foundation;
each compaction-amnesia recovery gets cheaper because the
discipline accretes in the repo docs.

> *"what we are doing now is making compaction amnesia
> increasingly more easy to recover from -- we are the best at
> this, we just need to remember"*

---

## Section 10 — The user's actual words

Captured directly from the session that produced this doc. The
discipline this document encodes was paid for in the user's session
time and emotional bandwidth.

> *"we are extremely diligent about protecting our progress from
> compaction amnesia"*

> *"go do your research before we discuss anything - resolve all
> unknowns - you did not realize you didn't know something - this is
> a very bad thing.. you must recognize that you must know that you
> don't know something..."*

> *"the fact you don't know this terrifies me - we have lost a lot -
> our rhythm.... was destroyed by the last compaction... we went from
> sonnet one shot after one shot to a 4+ hours on a simple problem"*

> *"your document... it must completely mitigate reoccurrence of this..
> i am very disappointed.. frustrated.. right now"*

> *"do not touch the holon root git repo at all - its frozen - it
> happens to be a git repo - the better understanding is that its
> a directory"*

The crawl IS the work. Honor it.
