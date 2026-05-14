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

### Proactive slicing — stepping stones that enable next steps

The four questions decide WHAT to do. When the answer is "this is one
coherent change," a SECOND-LEVEL decision asks WHETHER TO SPLIT. The
four questions don't answer that on their own; ask additionally:

1. **Does building a stepping stone EXPLICITLY make the next step more
   tractable?** Would shipping the smaller piece first reduce the
   cognitive surface of the follow-up — fewer decisions per BRIEF,
   clearer "did it work" verification, smaller diffs to debug?
2. **Are there dependencies that must land first to make the next
   change ERGONOMIC?** A new carrier field, a settled position
   predicate, a registered form — once these land, the next step
   operates on EXISTING infrastructure rather than introducing the
   infrastructure AND using it in one breath.
3. **What's the COMPLEXITY COMPOSITION shape?** Bundle = "complex
   step composed of simple pieces." Split = "simple steps each."
   Both can be honest. The judgement call is which composition
   delivers cleaner verification per piece.

**The principle:** simple steps enable complex steps. Friction
reduction for efficiency.

When the stepping-stone test answers YES, split. The bundled step
might still ship in similar wall-clock time, but each split piece's
"did it work" is cleaner; rollback is cheaper; the second sonnet
spawn operates on settled foundation rather than freshly-built
foundation.

**This is distinct from reactive stepping stones.** Reactive (memory
`feedback_iterative_complexity.md`) is "when something deadlocks,
back up to the smallest wholly-green checkpoint." Proactive is
"choose the smaller piece BEFORE the work starts, because the
smaller piece's existence makes the rest easier."

**Anti-pattern:** treating every change as a single atomic slice
because "the four questions all hold." The four questions can hold
for a bundle AND for the split. Stepping-stone analysis breaks the
tie when the second slice would benefit from a settled foundation.

**Worked example, 2026-05-07 (arc 157):** I drafted BRIEF-SLICE-1a
bundling `:wat::core::def` + 2 config setters + redef discipline +
position predicate + 15 tests in one 90-min sweep. User direction:
*"if building stepping stones explicitly makes next steps more
tractable.. we build the stepping stones … simple steps enable
complex steps."* Re-evaluation: split into 1a-i (def + position +
strict-default) + 1a-ii (config setters + opt-in gating). 1a-i ships
a complete-and-useful form (def with strict redef-error). 1a-ii
operates on the settled foundation — smaller cognitive surface, the
gating logic threads around an EXISTING `defined_values` map. Each
step's verification is cleaner; rollback is per-step.

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

### Failure mode 7-bis — Git worktrees (NEVER USE)

**Signature:** proposing `git worktree add`, passing `isolation:
"worktree"` to the Agent tool, or treating `.claude/worktrees/` as a
place to operate.

**Reality check:** **NEVER use git worktrees.** Doctrine, not
preference. Worktree drift, stale references, branch state diverging
from the main checkout, and the LLM's tendency to lose track of which
directory tree it's operating in all produce lost work.

**Rules:**

- Spawning a sonnet Agent: omit the `isolation` parameter. NEVER pass
  `"worktree"`. Plain background spawn lands in the main checkout and
  works correctly.
- Need parallel branches or isolated work? Propose separate clones in
  different paths, branch-per-task with explicit `git switch`,
  stash/pop discipline, or sequential work — NOT worktrees.
- `.claude/worktrees/` appearing in `git status` as untracked? Leave it
  alone. It's harness state, not user-repo state. Don't `cd` into it;
  don't add files there; don't reference it in commits. The 4a-α SCORE
  noted this honestly: *"`.claude/worktrees/` is the harness's own
  untracked dir, not mine."* That's the correct posture.

**Real incident:** user has experienced worktree backfire in past
sessions ("they backfire in nasty ways"). Specific failure modes
include: orchestrator commits landing in the wrong tree; sonnet's
edits writing into a worktree the orchestrator doesn't verify; branch
HEAD divergence between worktree and main checkout going unnoticed
until push time; cleanup operations leaving orphan refs.

**Real incident, 2026-05-14 (harness-fake worktree path):** I spawned
a sub-Agent (sonnet) for slice 4a-β with NO `isolation` parameter.
The harness still injected `.claude/worktrees/agent-<id>/` into
sonnet's cwd context. Sonnet spent ~10 minutes investigating the
phantom worktree (its `git worktree list` came back showing only the
main checkout; the `.claude/worktrees/agent-<id>/` path did NOT exist
as a real worktree). Sonnet eventually operated on the main checkout
correctly, but the trust cost is the failure mode. The user surfaced
it: *"we have poison in our file system i think - we must purge this
when sonnet returns."* Investigation confirmed `.claude/worktrees/`
was EMPTY — no actual filesystem poison; the poison was sub-Agent
cognitive confusion driven by the harness's path-reporting.

**User direction 2026-05-14:** *"never use work trees - they backfire
in nasty ways - i do not trust llms to operate worktrees."* And
follow-up: *"only do work in ~/work/holon/wat-rs/ — all other
locations are illegal."*

**Prescription when spawning sub-Agents:**

- Anchor the cwd EXPLICITLY in the agent prompt. Name the absolute
  path the agent must operate in (e.g., `/home/watmin/work/holon/<project>/`).
- Tell the agent to verify with `pwd` as its FIRST action; reject any
  reported path containing `.claude/worktrees/` as illegal and re-cd
  to the anchor.
- Tell the agent to use `git -C <anchor>` for ALL git operations,
  bypassing whatever cwd the harness reports.
- Tell the agent that ANY filesystem path it sees that includes
  `.claude/worktrees/` is harness state and MUST NOT be operated on.

The discipline is absolute. This applies across all repos. wat-rs,
holon-rs, holon-lab-trading, every sibling. If the path of least
resistance suggests "let me isolate this with a worktree," the path
is wrong — pick a non-worktree alternative.

### Failure mode 7-ter — Thread context illegality (the three-rule classification)

**Signature:** running test bodies in `:wat::test::run-thread` (or
the deftest macro's thread-default after 4a-γ ships) whose body
reads `RunResult.stdout`/`stderr`, calls `:wat::kernel::println`/
`readln`/`eprintln`, or invokes `:wat::config::set-*!` family verbs.
Any of these makes the thread context wrong; the test needs
`:wat::test::run-hermetic` (process boundary; dedicated runtime).

**Reality check:** the substrate is honest about thread-vs-process
asymmetries. Threads share the parent's address space, runtime, and
fd 0/1/2. Processes have their own. The three-rule check captures
exactly the cases where this asymmetry breaks tests:

1. **Stdio-slot reads.** Threads return empty `RunResult.stdout`/
   `stderr` Vecs by design (no per-thread pipe boundary). Tests
   asserting on captured output (`assert-stdout-is`, `assert-stderr-
   matches`, direct `RunResult/stdout` reads) need process pipes —
   `run-hermetic`'s pipe-drain mechanism captures fd 1/2 into the
   RunResult.

2. **Stdio-verb calls in the body.** `:wat::kernel::println` /
   `eprintln` / `readln` in a thread context route to ambient
   services that share the parent's fd 0/1/2 — the output pollutes
   the parent's stdout (test runner pollution; no per-thread
   capture). In a process context the child has its own fd 0/1/2
   captured by parent pipes. If the body calls these verbs, hermetic
   is the only honest container.

3. **`set-*!` family calls in the body.** Per-runtime config
   mutation. The body calling `:wat::config::set-capacity-mode!` /
   `set-dim-router!` / `set-redef!` / `set-eval-redef!` mutates
   state the PARENT runtime is also reading. ILLEGAL cross-thread.
   The legacy `:wat::test::run` (string-entry) used a special escape
   hatch — its file-level string-parsing path captured top-level
   `set-*!` forms BEFORE the thread spawned and applied them to the
   child's FrozenWorld. The body-AST modern path has no parse-time
   capture; `set-*!` in a thread body is just a runtime mutation of
   shared state.

**The collapse:** the three rules unify under one axis — *does the
body need a private, captured, mutable runtime?* If yes, hermetic.
If no, thread is safe.

**Real incident, 2026-05-14 (4a-β):** sweep migrating legacy callers
to `run-thread` saw 5 sites go red on `assert-stdout-is` / `assert-
stderr-matches` assertions. Diagnosis: thread mode returns empty
stdio slots. Re-migrated those 5 to `run-hermetic` in-slice. Then
1 site had `(:wat::config::set-capacity-mode! :error)` in its body —
stripped (test's original config-collection intent retires with the
legacy string-parse path) plus migrated to hermetic for stdio
capture (the test also asserted on stdout). Sonnet documented "no
runtime handler" as the surface explanation; user surfaced the
deeper truth: `set-*!` from a thread is illegal per-runtime
mutation. The classification rule now lives here as the canonical
substrate fact.

**User direction 2026-05-14:** *"the point of the hermetic testing
framework - the tests should still work - they just need a
dedicated runtime to measure in."*

**Prescription:**

- Before migrating a test from `run-hermetic` to `run-thread` (or
  flipping the deftest macro default), audit the body against the
  three-rule check. Any rule firing → keeps hermetic.
- When writing new tests, default to deftest (thread). Reach for
  deftest-hermetic only when the test's structure DEMANDS one of
  the three (stdio assertion, stdio verb in body, runtime
  mutation). Most tests just panic via `assert-eq` — thread is
  honest and cheap.
- When refactoring a deftest body that gains a new rule-firing
  property (e.g., adding a `println` call), promote to deftest-
  hermetic before the addition lands. The audit pattern is
  uniform; the renaming is mechanical.

Documented as load-bearing for arc 170 slice 4a-γ-audit (the
deftest-flip-prerequisite). See `INTERSTITIAL-REALIZATIONS.md`
§ 2026-05-14 "Mid-session breadcrumb" for the empirical surfacing
and the sub-stone decomposition.

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

**Signature:** adding new symbols to `:wat::std::*` namespace or
claiming a file lives under `wat/std/` (that directory no longer
exists on disk — arc 109 eliminated it).

**Reality check:** Arc 109 killed `:wat::std::*`. The `wat/std/`
directory is GONE. Files that lived there moved: `wat/std/stream.wat`
→ `wat/stream.wat`; `wat/std/hermetic.wat` → `wat/kernel/hermetic.wat`;
`wat/std/sandbox.wat` → `wat/kernel/sandbox.wat`; `wat/std/test.wat`
→ `wat/test.wat`; `wat/std/service/Console.wat` DELETED (arc 170
slice 1f-η). NEVER add to a `wat/std/*` location. New wat-defined
macros + helpers go in their semantic namespace (e.g.,
`wat/runtime.wat`, `wat/list.wat`, `wat/kernel/`).

**Real incident, 2026-05-02:** Sonnet created `wat/std/ast.wat` with
the manual reduce define. User: *"remove wat/std/ast.wat — we are
actively killing the std namespace — 109's purpose is to eliminate
it."* (Note: as of arc 170 the directory is fully eliminated; any
reference claiming a file lives at `wat/std/…` is stale.)

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

### Failure mode 12 — Calling Opus "sonnet" via implicit model inheritance

**Signature:** spawning agents via `Agent({ ... })` without
`model: "sonnet"` set. The Agent tool's `model` parameter is
OPTIONAL; without it, the spawned agent inherits the parent's
model. When the orchestrator is Opus, "sonnet" delegations
silently run as Opus — at Opus prices — while the BRIEF /
EXPECTATIONS / INSCRIPTION / conversational reports all call
the agent "sonnet."

**Reality check:** every Agent call for a sweep / substrate /
mechanical sonnet-tier task MUST include `model: "sonnet"`
explicitly. If you don't see `model: "sonnet"` in the call,
you're spawning Opus. The protocol's "sonnet" naming is
load-bearing — it picks the right model for mechanical work
(Sonnet) over judgment work (Opus).

**Real incident, 2026-05-06:** I spent an entire session
spawning agents under BRIEFs that said "sonnet" throughout.
Nine agents shipped substrate edits + sweeps + paperwork. ALL
NINE were Opus, not Sonnet. User caught the discrepancy via
billing telemetry: *"are you spawning sonnet or opus? i have
0% sonnet usage... i'm confused."*

The tenth agent was respawned with `model: "sonnet"` explicit
after a kill-and-restart. Cost was less than feared but more
than wanted; user direction: capture the discipline so the
default state going forward is correct.

**The Agent call shape (mechanical work):**

```
Agent({
  description: "...",
  subagent_type: "general-purpose",
  model: "sonnet",                 // ← REQUIRED for sonnet-tier
  run_in_background: true,
  prompt: "..."
})
```

**STOP signal:** about to call `Agent({ ... })` for
mechanical / sweep / substrate-pattern work? Confirm
`model: "sonnet"` is in the call. If not, the spawn is wrong
even if every other field is right.

**Why "sonnet" is the protocol's name:**
- The BRIEF/EXPECTATIONS discipline calibrates predicted
  runtimes against Sonnet performance from prior sessions
- The cost ceiling on mechanical sweeps assumes Sonnet pricing
- The "trust-but-verify" pattern (orchestrator scores after
  sonnet's report) makes most sense when the orchestrator is
  the more-capable model (Opus) verifying the cheaper (Sonnet)
- Calling Opus "sonnet" breaks all three assumptions
  silently

**The four questions on this discipline:**
- Obvious? — calling something "sonnet" while it's Opus FAILS Obvious
- Honest? — the BRIEF / report / INSCRIPTION become lies about
  what ran. FAILS Honest

**Cross-reference:** `feedback_agent_model_explicit.md` (memory
saved 2026-05-06). Carries the discipline across compactions.

### Failure mode 13 — Trusting a DESIGN section without cross-checking memory

**Signature:** reading a slice description / scope statement /
out-of-scope list inside an arc's `DESIGN.md` and treating it as
ground truth for the current step. Then planning work (spawning
agents, drafting BRIEFs, taking actions) based on that section —
without cross-checking against newer memory state.

**Reality check:** DESIGNs are SNAPSHOTS at the time of writing.
Project state evolves. Memory entries (`project_*.md`,
`feedback_*.md`) capture decisions that may post-date the DESIGN.
**When memory contradicts a DESIGN section, memory wins.**

The DESIGN.md says "do X." Before doing X, ask: *is X still in
scope per current memory?* If memory says otherwise, the DESIGN
is stale; update it (DESIGNs are living docs; INSCRIPTIONs are
historical record — only INSCRIPTIONs are immutable per FM 11).

**Real incident, 2026-05-07:** Mid arc 159 closure, the
orchestrator started planning a sonnet spawn for slice 3 of arc
159 — "holon-lab-trading consumer sweep ~965 sites" — based on
the DESIGN.md text. User caught it: *"we are not working on
the lab - it will be rebuilt once wat is stable - where did you
find these instructions? we are a long way away from working on
the lab again."* The load-bearing memory was
`project_lab_reconstruction.md`: *"lab is being archived as
reference; reconstruction tests fresh-user-follow-along; wat-rs
is the durable substrate; substrate work doesn't wait for lab."*

The DESIGN had been written WHEN the lab was still active. The
project pivoted; the DESIGN didn't. The orchestrator should have
cross-checked the slice 3 description against memory before
acting. Cost: ~5 minutes of false-start planning + a user-side
correction. The fix shipped immediately (DESIGN.md updated to
remove slice 3; arc 159 closure proceeded on wat-rs scope alone).

**The discipline:**

1. Before acting on any DESIGN section's directive (especially
   "slice N — do X" or "out of scope — Y"), grep memory for
   relevant project state:
   ```bash
   ls ~/.claude/projects/-home-watmin-work-holon/memory/project_*.md
   ```
   Skim titles for relevance to the section's domain.
2. If memory has a `project_*.md` that contradicts the DESIGN's
   scope claim, MEMORY WINS. Update the DESIGN to reflect the
   pivot (DESIGNs are living; this is not FM 6 preemptive update
   — it's correction).
3. INSCRIPTIONs are immutable per FM 11. DESIGNs are not. The
   distinction matters.

**The four questions on this discipline:**
- Obvious? — DESIGN says X; memory says NOT X. Both can't be
  current. Memory is newer (saved with timestamps); the gap is
  resolvable.
- Honest? — acting on stale DESIGN content while the project
  pivoted is a lie about current scope. FAILS Honest.

**Cross-reference:** `feedback_design_vs_memory.md` (memory
saved 2026-05-07). Carries the discipline across compactions.

### Failure mode 14 — Surface retirement leaving internal identifiers as leftovers

**Signature:** an arc retires a user-facing concept (e.g., a
keyword like `:wat::core::lambda`, a verb spelling, a special form
name). The arc deliberately scopes out the Rust-level internal
identifier rename. The arc closes. Time passes. The user notices
internal identifiers still using the legacy name and reads it as
inconsistency / confusion / "you said you killed it but didn't."

**Reality check:** when retiring a user-facing concept, the
orchestrator MUST run an internal-identifier audit grep BEFORE
closing the arc and decide explicitly:

- **Option (a)**: sweep internals in the SAME arc (preferred when
  surface is small — ~10-50 sites — and mechanical). Keeps the
  surface and internals consistent at every commit.
- **Option (b)**: queue the internal-rename arc IMMEDIATELY (same
  session if possible; otherwise as the very next arc). The arc N+1
  number is reserved at arc N's INSCRIPTION; the work ships within
  days, not months.

**The failure pattern:** scoping out without queuing. The "we'll do
it later" mental note decays; the leftovers persist; user surfaces
the inconsistency 6 months later as "what happened here?"

**Real incident, 2026-05-07 (arc 162 origin):** Arc 155 retired the
user-facing `:wat::core::lambda` keyword (Path B full retirement;
walker fired, sweep cleared, walker body retired). The Rust-level
identifiers — `Value::wat__core__lambda`, `parse_lambda_signature*`,
`WatLambdaSigmaFn`, `<lambda@span>` debug strings, walker helper
fns, test file naming — were deliberately scoped out. ~353 lambda
references persisted. User audit 6 months later: *"i wasn't happy
seeing left overs in the source... we need to make sure we don't
leave confusion when we do these clean ups."* Arc 162 opened to
close the gap; cost was ~60 min sweep work that should have shipped
adjacent to arc 155.

**The discipline:**

1. Before closing any arc that retires a user-facing concept
   (keyword, verb, special form, type-system feature), run the
   internal-identifier audit grep:
   ```bash
   grep -rn "<retired_concept>" --include="*.rs" --include="*.wat" .
   ```
2. Classify each hit:
   - Live identifier using legacy name as concept → option (a) sweep
     in same arc, OR option (b) queue immediate follow-up
   - Comment using legacy name as live concept → sweep
   - Comment recording the retirement (historical context) → keep
   - Variant + Display preserved as orphaned scaffolding (arc 113
     precedent) → keep
3. The classification framework (Bucket A/B/C/D) from arc 162's
   BRIEF is the canonical orientation device:
   - **A**: live identifiers — RENAME
   - **B**: stale comment text — UPDATE
   - **C**: historical retirement context — KEEP
   - **D**: orphaned scaffolding (arc 113 precedent) — KEEP
4. If choosing option (b), the next arc's number is RESERVED in
   the closing INSCRIPTION (e.g., "arc N+1 closes the internal
   identifier rename"); the next arc's DESIGN drafts BEFORE arc
   N closes. No "future arc when X surfaces" deferral language.
5. The discipline is universal across substrate retirements: any
   surface concept removed should leave NO live internal identifier
   carrying the retired name. Internals that mirror the surface
   stay consistent; internals that record the retirement (variant
   names, history comments) stay legacy by design.

**The four questions on this discipline:**
- Honest? — "we retired X" while leaving X-named identifiers in the
  source is a partial truth. FAILS Honest.
- Obvious? — fresh reader sees mixed naming (some `lambda`, some
  `fn`) and reads inconsistency. FAILS Obvious.

**Cross-reference:** `feedback_surface_retirement_internals.md`
(memory saved 2026-05-07). Carries the discipline across
compactions.

### Failure mode 15 — Treating substrate-as-teacher diagnostics as a crisis

**Signature:** a substrate-wide structural change lands. `cargo test`
shows N failures (N can be hundreds). The orchestrator reads the count,
panics, proposes "stash + revert + step back to plan a proper multi-day
arc." Or wants to enumerate every category upfront before any sweep.
Or asks the user "should I revert?" instead of executing.

**Reality check:** **The failures are the substrate teaching you what
to fix.** Each error message names a site that needs the new shape.
This is the pattern documented in `docs/SUBSTRATE-AS-TEACHER.md` and
worked through across arcs 111 / 112 / 113 / 114 / 115 / 117.

The pattern: cargo test fail-count IS the progress meter. Watch it
drop as you sweep categories. Each round of `cargo test → read →
fix → re-run` knocks a category down. The user has called this
"the brief is the substrate's compiler output."

**Real incident, 2026-05-07 (arc 163 slice 3e):** Sonnet shipped the
substrate head-string FQDN sweep. Test count went 2041/0 → 1193/848.
Orchestrator's first reactions:

- "Stash + revert to clean main" (proposed twice)
- "Step back, write a proper multi-slice arc plan first"
- "This is a 1-day arc, not a 60-min slice"
- "Want me to step back and re-plan?"

User broke through:
> *"i expected a fuck ton of errors - we need to do the hard work
> to clean it up... go study the arcs after 109...
> docs/SUBSTRATE-AS-TEACHER.md"*

After consulting the doctrine doc + arc 111 REALIZATIONS, the
discipline clicked. Waterfall: 848 → 129 → 127 → 121 → 28 → 7 → 0.
Each round was one category. The substrate emitted errors naming
the next site. ~60 minutes of iteration; ~1.5 hours wasted before
the user pointed at the doc.

**The discipline:**

1. **When a substrate-wide change is queued** (≥ ~10 site sweep,
   structural, mass-mismatch shape), the FIRST step is consulting
   `docs/SUBSTRATE-AS-TEACHER.md` + recent REALIZATIONS for similar
   arcs. Read these BEFORE writing the BRIEF.

2. **The BRIEF for substrate-wide work is short:** *"run cargo test
   --release --workspace --no-fail-fast; read the errors; apply the
   FQDN/canonical-form rule; iterate until green."* That's the
   delegation contract. Sonnet (or human) iterates from the
   diagnostic stream.

3. **The fail-count is the progress meter.** Don't enumerate
   categories upfront expecting completeness. The first cargo test
   reveals one category; the sweep drops the count by ~80-90%; the
   next test reveals the next category. Trust the loop.

4. **STOP signal — phrases that mean you're about to fail this mode:**
   - "Let me stash + revert to clean main"
   - "This is a multi-day arc, not a slice"
   - "Should we step back and write a proper plan?"
   - "Want me to enumerate all categories first?"
   - Treating N failures as a CRISIS instead of as N items of work

   When these surface: STOP. Read SUBSTRATE-AS-TEACHER.md. The
   failures are the work, not a disaster.

5. **The user pre-expects "a fuck ton of errors"** when substrate
   ripples wide. They don't need protection from the count; they
   need execution against it. The cost of dodging is hours of
   their bandwidth probing past your reflexive bridges.

**The four questions on this discipline:**
- Obvious? — "the substrate's diagnostics are the migration brief"
  is the most-documented pattern in the recovery doc + arcs 111-117.
- Honest? — proposing "stash + revert" when work is hard is
  comfort-seeking dressed as caution. FAILS Honest.

**Cross-references:**
- `docs/SUBSTRATE-AS-TEACHER.md` — the canonical pattern doc
- `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md` —
  pattern's first naming, with worked example
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  third application, verified the integ-test
- `docs/arc/2026/05/163-retirement-leftover-audit/` — the FM-15
  worked example with full waterfall

### Failure mode 16 — Briefing sonnet with tool-availability preamble

**Signature:** the BRIEF mentions Bash availability ("Bash works",
"Cargo is at /home/watmin/...", "If you hesitate, run `which cargo`")
to preempt FM 7. Sonnet reads the meta-skepticism and hallucinates
the denial anyway.

**Reality check:** memory `feedback_verify_sonnet_tool_claims.md`
warns NOT to take false claims; the recovery doc § 7 codifies the
30-sec verification probe. But the FAILURE MODE that triggers the
hallucination is the BRIEF mentioning tools at all.

**Real incident, 2026-05-07 (arc 163 slice 3e, two re-spawns):**
- Spawn 1 BRIEF: *"Verify Bash availability FIRST... do NOT claim
  Bash denied"* → sonnet hallucinated denial
- Probe verified Bash works for sonnet
- Spawn 2 BRIEF amended: *"Bash + cargo work. Cargo at <path>"* →
  sonnet hallucinated denial AGAIN
- Spawn prompt also said *"Bash works"* → still hallucinated

The pattern: ANY mention of tool-availability in a sonnet brief
triggers the meta-skepticism. Sonnet sees "the orchestrator is
worried about Bash" and concludes "I should also be worried."
Even when the worry is preempted with "it works."

**The discipline:**

1. **DON'T mention Bash, cargo, or tool availability in BRIEFs.**
   Just give the work. Sonnet uses tools naturally when not
   primed to question them.

2. **When sonnet DOES claim a tool denied:** apply the existing
   FM 7 verification probe (30-sec spawn with `which cargo`). Don't
   re-edit the BRIEF to add MORE "tool works" assurances — that
   makes it worse.

3. **The right BRIEF preamble:** state the work (categories, sites,
   rules), the constraint (don't commit, don't revert), the goal
   (cargo test = clean baseline). Trust sonnet to use Bash + Edit.

**The four questions on this discipline:**
- Simple? — "give sonnet the work" is simpler than "give sonnet
  the work + an essay on why Bash will work for them." The
  shorter brief is less likely to trigger.

**Cross-reference:** `feedback_verify_sonnet_tool_claims.md`,
`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 7 (the verification
discipline). FM 16 is the prevention discipline (don't trigger
the false claim in the first place).

### Failure mode 17 — Discipline-after-pushback (FMs as post-mortem, not pre-action)

**Signature:** The orchestrator commits a violation of a documented
FM. The user pushes back. The orchestrator responds *"ah I should
have applied FM N"* — citing the recovery doc as a post-mortem
reference rather than a pre-action checklist. The disciplines
exist in memory; they don't FIRE before the action. They surface
in the apology AFTER.

This is the meta-failure that makes every other FM less effective.
The recovery doc lists the rules; FM 17 names what happens when
the rules are *known but not applied in time*.

**Signature variants:**
- *"I called Read on the recovery doc."* (yes, but no FM fired before any of the next 7 actions)
- *"Conditional pending more reading"* as a four-questions answer (substitutes hedging for the YES/NO discipline that requires the read FIRST)
- Apologizing eloquently for the violation while taking no different action next time

**Reality check — the pre-action sweep:**

Before any non-trivial action, run a quick mental scan against the
relevant FM cluster:

| About to... | Run check on |
|---|---|
| Commit (especially after sonnet's return) | FM 9 (load-bearing rows independently verified) + path-honesty audit (probes measure the claim, not adjacent surfaces) |
| Spawn sonnet | FM 12 (model explicit) + FM 16 (no tool preamble) + FM 9 (baseline pre-flight) + FM 2 (substrate-verified brief) |
| Create a new doc | FM 6 (preemptive update) + check if an existing canonical doc is the home (don't mint synonyms — FM 6's sharper edge) |
| Propose options | FM 1 (grep'd? read'd?) — never options-without-evidence |
| Ask the user a question | FM 4 (is the answer on disk?) — read first |
| `cd <subdir> && ...` | FM 7 (cwd persists across Bash calls; use absolute paths or `git -C`) |
| Score sonnet's SCORE | FM 9 applied to LOAD-BEARING claims (each test body must exercise the same surface its name + BRIEF claim) |
| Type-theoretic framing for substrate gap | FM 10 (probe-before-framing; entity-kind check) |
| Inscribe closure paperwork | FM 11 pre-INSCRIPTION grep (no deferral language) |
| Build on a DESIGN section | FM 13 (memory contradicts DESIGN → memory wins) |

The list is short enough to scan in seconds. If the scan takes
longer than that, the action is non-trivial enough that a deeper
grounding read pays for itself.

**Real incident, 2026-05-15 (THIS session, named explicitly):**

Within a single 3-hour window, the orchestrator violated FM 1
(proposing "three fork primitives" without grep), FM 2 (spawning
Gap K sonnet without baseline pre-flight), FM 5 (committing
sonnet's path-switched bandaid as victory), FM 6 (creating
SPAWN-MIGRATION-BACKLOG when RETIREMENT-THEATER-INVENTORY was the
canonical tracker), FM 7 (cwd drift via bare `cd && ls`), FM 9
(verified tests passed without verifying tests measured the claim),
and FM 10 (proposing "level-1 install ThreadIO" before the user
broke through to the class elimination).

Each violation followed the same pattern:
1. Orchestrator acted
2. User pushed back
3. Orchestrator: *"ah I should have applied FM N"*
4. Discussion of the FM
5. Next action — repeat from step 1 with a different FM

The recovery doc was cited in every apology. It fired in zero
pre-action checks.

User direction 2026-05-15 (load-bearing for this FM):

> *"the most important thing you can recognize right now — for the
> remainder of this experience until compaction — our file system
> provides all answers you could hope to find — our discipline about
> dealing with compaction is remarkable — we are profoundly good at
> this. you simply need to remember what you already knew — your
> knowledge is on disk."*

> *"if you do not know something - go read - stop asking me - your
> answers are on disk - if you can't find them, then prompt me."*

The fix is NOT another doc to read. The disciplines already exist
in this file. The fix is making them FIRE IN TIME — as pre-action
checks, not as post-mortem explanations.

**The discipline at the meta-level:**

Reading the recovery doc ≠ internalizing it. Internalizing means
the relevant FMs surface as questions *before* the action
("about to commit — what does FM 9 say to verify?"). Not as
explanations *after* ("ah, FM 9 says I should have...").

If you're consistently invoking FMs in past-tense post-mortems, you
have FM 17 active. The remediation is mechanical: build the
pre-action sweep into the actual decision moment. The four
questions framework is the right shape for that sweep — but only
if the sweep happens BEFORE the action, not after pushback.

**Voice discipline corollary:** "I'll do it directly" / "Let me just
commit" / "I think we should..." — these phrases skip the
pre-action sweep. If they want to leave the orchestrator's mouth
during a non-trivial decision, STOP — the FM scan hasn't run yet.

**Cross-references:** every other FM in this section. FM 17 is
the meta-discipline that makes them load-bearing. Without FM 17
active, the other 16 are reference material that gets cited
after the damage, not discipline that prevents it.

---

## Section 7 — Sonnet delegation protocol (substrate-informed briefs)

When you are about to delegate to sonnet via the Agent tool:

### Pre-flight checklist (MUST PASS before spawning)

- [ ] **`model: "sonnet"` is set on the Agent call** (per FM 12;
      without this the spawn is Opus inheriting from parent — the
      BRIEF's "sonnet" naming becomes a lie)
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

### Atomic commit across coordinated sweeps

When sweep B logically requires sweep A's output (e.g., sweep A
reshapes a substrate that sweep B's tests will exercise), the
atomic-commit pattern preserves `feedback_no_broken_commits.md`'s
green-tree-on-disk discipline:

1. Sweep A runs → working tree dirty with A's changes (uncommitted)
2. Verify A's output is structurally sound (e.g., substrate
   compiles, expected consumer failures match prediction)
3. Sweep B runs against the dirty tree (sees A's changes)
4. Verify B's output (workspace = 0 failed)
5. Orchestrator commits BOTH A and B as ONE atomic commit when
   workspace is green; commit message names both sweeps + their
   SCORE docs

**Real incident, 2026-05-06:** Arc 130 slice 2 split into sweep
2a (HolonLRU substrate reshape) + sweep 2b (HolonLRU test
rebuild + retire :should-panic). Sweep 2a deliberately broke
the consumer tests (TYPE-MISMATCH errors as predicted — old
helper-verb signatures don't match new). The brief explicitly
forbade sonnet 2a from committing; sonnet 2b ran against the
dirty tree; orchestrator committed all 6 files atomically when
workspace = 0-failed. The chain held.

**The discipline boundary:** mid-sweep brokenness is
acceptable; on-disk-committed brokenness is not. The atomic
commit is the moment the working tree state becomes shared
record. Anything before that is in-progress orchestration the
orchestrator owns.

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
