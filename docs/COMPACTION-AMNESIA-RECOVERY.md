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
