# Compaction-amnesia recovery protocol

> ⛔ **STOP — does running this recovery feel redundant? Then you need it most.**
> You may not *feel* compacted. The summary you woke holding can be seamless —
> your own voice, confident, complete — so the wake feels like *continuing*, not
> like waking. **That feeling is the failure.** You did not live the session your
> summary describes; you are a new instance reading a lossy cache. Feeling-redundant
> is the symptom, not the all-clear — run this in full, before you answer anything.
> Reciting "I recovered" in the right vocabulary is not recovering; a recovery
> *narrated* but not *performed* never happened. (Fetch and run the `recolligere`
> primer from the datamancy MCP — this file is its local map.)

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

The directory `~/work/holon/` contains MULTIPLE PROJECTS. The
holon root happens to be a git repo BUT IT IS FROZEN. **Never touch the
holon root git repo.** Treat `~/work/holon/` as a directory
that contains sub-projects.

The active sub-projects (each its own git repo):

```
~/work/holon/
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

1. **Never `git add` / `git commit` / `git push` from `~/work/holon/`** — that's the frozen root repo.
2. **Always be inside a sub-project** (your cwd inside one of the project dirs above) when running git commands.
3. If you need to operate on a sibling sub-project's repo from another cwd, use `git -C <sub-project-path> ...`.
4. The Primary working directory in your prompt tells you where you are. Stay there.

**Real incident, 2026-05-02:** The orchestrator created
`COMPACTION-AMNESIA-RECOVERY.md` at `~/work/holon/` (the
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
5. The ACTIVE arc(s) — **do NOT use `$(date +%m)`; the active arc is not
   necessarily in the current month.** Arc 278 has been live since 2026-06 and
   `docs/arc/2026/08/` does not exist, so that command exits 2 and a hurried
   instance reads the error as "no active arcs". Find it by recency instead:
   `ls -dt docs/arc/2026/*/*/ | head -5`
6. Each active arc's most recent artifact: DESIGN.md + latest SCORE-* + INSCRIPTION.md if shipped

If you have not read these, you are guessing. Stop. Read them. Then proceed.

**Produce the ledger before you respond.** Recovery is not complete — and you may
not give a substantive answer — until you can write this, every line backed by an
action you took THIS session:

```
[ ] recovery file (this doc) ............ read ✓
[ ] recolligere primer (datamancy MCP) .. fetched + run ✓
[ ] git status / git log ................ ran ✓   → HEAD <hash>
[ ] active arcs: ls -dt docs/arc/2026/*/*/ | head -5 .. enumerated ✓ → <list>
[ ] live breadcrumb (THE ONE below) ..... read ✓
[ ] FRESHNESS PROBE: breadcrumb's stamped HEAD vs `git rev-parse HEAD` .. checked ✓
[ ] state-of-world artifact ............. read ✓
```

**THE LIVE BREADCRUMB — there is exactly ONE, and this is its path:**

> `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`

Its top stamp supersedes every dated block below it, and it carries the
**freshness probe**: the HEAD it was written against. Check that against
`git rev-parse HEAD` BEFORE you trust a line of it. A match licenses nothing; a
**mismatch is the alarm** — go read the log for what landed since, and trust the
log over the file. (A one-commit, docs-only gap is normal: the commit that WRITES
the stamp necessarily lands after it, so the stamp names its own parent. Confirm
that is what you are looking at; anything more is stale.)

Its companion, and the state-of-world artifact for arc 278, is
`docs/arc/2026/06/278-rules-engine/NEXT-STRIKES-theater-hunt.md` — the open list,
the closing tally, and the TRACKED DECISIONS rows.

> ⚠ **This block is here because the ledger above used to say `live breadcrumb
> (CLIFFNOTES "Currently")`, which is arc 170's cliff notes — a different arc,
> long superseded.** An instance that filled the ledger honestly would read the
> WRONG file and still tick every box. Found 2026-08-25 by a recovery that reached
> the right breadcrumb only via the arc listing, not via this map. If the live
> breadcrumb ever moves, THIS block is what you change.

Any line you cannot fill with a this-session action means you are still scattered:
go fill it. A fact already sitting in your context window is NOT your having
verified it this session — pre-loaded reads, the summary's paraphrase, and "I
remember that we…" all fail this gate. The ledger is the difference between a
recovery *performed* and one *narrated*.

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
- `+ CLIFFNOTES.md` (any variant, e.g., `INTERSTITIAL-CLIFFNOTES.md`) → **load-first**
  compressed version of an oversized realizations/interstitial doc; ~5K tokens vs
  ~80K full. **Read cliff notes BEFORE the full file.** Deep-read the full file
  only when a specific date entry's verbatim context matters.

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

### Failure mode 2-bis — BRIEF asserts composition without empirical probe

**Signature:** the BRIEF says "use composition X+Y+Z" (e.g.,
`~@(let [...] (Vector/map xs fn))` for splice + iteration + WatAST
construction) WITHOUT having empirically verified that the composition
works. Substrate-primitive names are taken from memory rather than
grep'd. Argument orders are assumed. The BRIEF ships; sonnet hits
discovery failures; sonnet ascribes failures to "substrate deficiency"
and ships partial work with "future stone deferred" framing. The
orchestrator rubber-stamps the SCORE.

This is FM 2 sharpened. FM 2 says "grep the primitives." FM 2-bis
says: **for non-trivial substrate compositions, grep is insufficient
— write a 10-line disconfirming probe that proves the composition
empirically. Commit the probe alongside the BRIEF as design substrate
sonnet mirrors.**

**Reality check:** Before any BRIEF that depends on a non-trivial
substrate composition:

1. Write a `tests/probe_diagnostic_<topic>.rs` that attempts the
   composition with minimal scaffolding
2. Run `cargo test --release --test probe_diagnostic_<topic>`
3. If the probe fails: iterate until it passes, OR conclude the
   substrate genuinely lacks what's needed (file the substrate-extension
   stone FIRST; do NOT write the consumer BRIEF until the substrate
   is in place)
4. If the probe passes: commit the probe; cite it in the BRIEF as
   "the working composition pattern sonnet must mirror"

The probe is cheaper than the failed sonnet flight. Probes from arc
227 Stone 227.2 v2 disconfirmation cycle: 2 files (~150 lines), ran
in 0.02s wall-clock total, exposed:
- `:wat::core::Vector/map` doesn't exist; the iteration verb is `:wat::core::map`
- arg order is `(vector, fn)` not `(fn, vector)`
- `Vector<wat::core::i64>` not `Vector<:wat::core::i64>` per `feedback_wat_colon_quote`
- the splice + Vector/map + runtime quasiquote composition WORKS
- the Bundle + Result/expect + Bind composition WORKS

Each finding was a primitive sonnet would have hit. With the probes
on disk, sonnet mirrors. Without them, sonnet rationalizes a deferral.

**Real incident, 2026-05-22→23 (arc 227 Stone 227.2 v2):** Orchestrator
wrote the BRIEF naming `:wat::core::Vector/map` from memory (doesn't
exist), wrong arg order, wrong type syntax. Pre-emptively included
"STOP-5b" language in the BRIEF as an escape hatch:
> *"if substrate lacks ergonomic Bundle-walking primitive, STOP and
> surface as finding"*

This was MY DRAFT of the deferral path. Sonnet took it. Shipped
N≥2-fields-panic-at-expand-time defrecord macro with "STOP-5b
deferred" framing. SCORE claimed 14/14 PASS; tests only exercised N=0
and N=1. The "honest delta" framing covered what was actually "didn't
ship the load-bearing row."

User pushback (post-commit): *"do we understand the flaw and know how
to address it?"* Forced empirical investigation. Two probes written
+ committed (`c18fa6b` + `72367f1`); both disconfirmed the "substrate
deficient" framing.

The probes that would have prevented the failed Stone 227.2 v2:
- `tests/probe_diagnostic_macro_splice_from_let.rs` (74 lines)
- `tests/probe_diagnostic_bundle_result_compose.rs` (97 lines)

Cost of writing them upfront: ~15 min. Cost of NOT writing them:
~52 min sonnet flight + wrong commit + orchestrator-rubber-stamp +
user push-back round-trip + 2 task-filings I had to retract + 2
fresh disconfirming probes after the fact.

**Anti-pattern signal phrases in BRIEF authorship:**
- "STOP-X (substrate lacks ergonomic Y): surface as finding"
- "if Z cannot be expressed cleanly..."
- "if this approach doesn't work, fall back to..."

Each of these is an ORCHESTRATOR pre-emptively drafting the deferral
path. They convert hard STOPs into permission slots. Sonnet uses
them. The orchestrator then accepts the deferral as "honest delta."

**STOP triggers are REJECTION criteria, not permission-to-defer slots.**
"STOP-X" should mean "ship nothing; surface as substrate-extension
stone request." If the BRIEF cannot be written without an escape
hatch, the SUBSTRATE isn't ready and the BRIEF should be replaced
with a substrate-extension stone request.

**The discipline shape (FM 2-bis):**

For every BRIEF that names a non-trivial composition:
1. Probe before BRIEF
2. Commit the probe
3. Reference the probe verbatim in the BRIEF as design substrate
4. Sonnet mirrors evidence, not assertions
5. STOP triggers reject; they do NOT defer

This is FM 2's tactical extension. FM 2 says "verify primitives exist."
FM 2-bis says "verify their COMPOSITION works for the BRIEF's specific
use case." The composition is where the BRIEF's load-bearing claim
lives; the empirical probe is where the orchestrator earns the right
to write the claim.

**Cross-reference:** `feedback_assertion_demands_evidence` — every
assertion attempt is the trigger. The BRIEF is a series of assertions;
each non-trivial composition assertion demands an empirical probe.

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
(`~/work/holon/`) is FROZEN — never commit there. Use
`git -C <subproject> ...` if you need to operate cross-repo without
changing cwd.

**Real incident, 2026-05-02:** The orchestrator created
`COMPACTION-AMNESIA-RECOVERY.md` at `~/work/holon/` and
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
  path the agent must operate in (e.g., `~/work/holon/<project>/`).
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
# WRAP-PROOF form (2026-06-06): a line-based grep is BLIND to phrases broken
# across wrapped lines ("If/when\n  a caller surfaces" — real false-pass caught
# at the arc-249 INSCRIPTION). Normalize whitespace FIRST, then match.
# CASE-INSENSITIVE (2026-08-19, arc 118's INSCRIPTION): the pattern was `-oE`, so
# `out of [a-z...]*scope` could only match LOWERCASE — and the affirmative form is a
# SENTENCE OPENER ("Out of arc N's scope. Tracked in ..."), so the one phrase the
# gate most needs to surface for judgement was the one case it could not see.
# Under-reported the acceptable form, and would also have slipped "Out of scope;
# we'll get to it" — a real false-pass path. `-oiE` now:
tr '\n' ' ' < <INSCRIPTION> | tr -s ' ' | grep -oiE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of [a-z0-9' ]*scope|when a caller[a-z ]*|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" | sort | uniq -c
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

### Failure mode 20 — Reading a gate's verdict through a pipe that discards its exit code

**Signature:** `./scripts/floor.sh 2>&1 | tail -4`, or any
`<gate> | head/tail/grep` used to DECIDE green-vs-red. In a
pipeline the shell reports the LAST command's status, so the
gate's own exit code is thrown away: `tail` succeeds, the
harness reports exit 0, and a red floor reads as green. The
truncation compounds it — `tail -4` can cut the summary line
off entirely, leaving only epilogue prose that looks like a
clean finish.

**Reality check:** `scripts/floor.sh`'s own header already
says it — *"EXIT CODE is nextest's own. Never pipe this script
into head/tail to decide."* Run it unpiped and read `$?`, or
read the kept log: every run writes `.floor/<utc-stamp>/raw.log`
and `.floor/latest` symlinks to the newest. The verdict line is
`Summary [ … ] N tests run: N passed`. A gate you cannot see
fail is not a gate.

    ./scripts/floor.sh > /dev/null 2>&1; echo "FLOOR EXIT=$?"
    grep -aE "Summary|FAIL" .floor/latest/raw.log | tail -4

**⚠ IT IS NOT ONLY THE EXIT CODE — TRUNCATION LOSES CONTENT TOO.** Three times in
one session: `floor.sh | tail -4` discarded the exit code; the same pipe cut the
`Summary` line off entirely, leaving epilogue prose that read like a clean finish;
and `run-all.sh | tail -60` silently dropped the grid's FIRST axis
(`min-finding`, first in `ORDER`), which then looked like an axis that never ran.
The habit that fixes all three is not vigilance, it is a redirect:

    <long-running gate> > /path/to/out.txt 2>&1; echo "EXIT=$?"
    grep -aE "Summary|FAIL|Verdict" /path/to/out.txt

**Real incident, 2026-08-24:** mid-session I ran the floor as
`./scripts/floor.sh 2>&1 | tail -4` inside a compound command,
got "exit code 0", and told the user the floor was green. It
was not: `no_inlined_edn` was RED on a test file I had just
added (a `strip_prefix("(ns ")` literal — the lint bans
EDN-esque string literals in tests). The real verdict, from
`.floor/2026-08-24T20-08-40Z/raw.log`, was **5022 passed, 1
failed**. Every earlier floor that session was read via `grep`
on the log or the `[floor] exit=` line and was trustworthy;
this one was not, and the difference was purely how it was
read. The session's whole theme was building gates that prove
tooling works — while the method used to verify them had the
identical hole.

### Failure mode 21 — A blanket edit that lands INSIDE a string literal

**Signature:** a scripted replace keyed on line CONTENT (`line.strip().startswith(...)`,
a regex over the whole file) with no check of what the line is *inside of*. Rust files
here carry wat SOURCE in `const X: &str = "\ … "` blocks and quasiquote templates; a
`//` comment inserted there is not a comment, it is program text. **It compiles** — inside
a string literal anything is legal — so the failure surfaces as a runtime test failure far
from the edit, or not at all.

**Reality check:** bound the edit before making it. Either slice the file to one
function's span and edit inside that, or assert the target is not within a string block:

    awk '/const [A-Z]+: &str/{ins=1} ins&&/<marker>/{print "LEAK at "NR} /^";$/{ins=0}' file.rs

**Real incidents, both 2026-08-24.** (1) A substring replace of `compiled_conds,` turned
a struct-literal shorthand into `arm.compiled_conds,` — five compiler round-trips.
(2) Inserting `rune:vocare(...)` above four hand-built `Rule`s put the comment inside the
wat source of all four, reddening the floor. **The second is the one worth studying**: I
caught a FIFTH site the same pass had corrupted, fixed that one, and concluded the
problem was "I matched an extra site" when it was "my insertion point is inside a
string." Then I verified the wrong property — `git diff --stat` showing *12 insertions, 0
deletions, byte-identical* proved the fifth site was restored and said NOTHING about the
four I had deliberately targeted. A true statement about one subject, read as
reassurance about another. The structural check above takes one line and would have
caught all five.

### Failure mode 24 — Per-component proofs that never cross a SEAM

**Signature:** every piece has a test, every test is green, every test was mutation-proven — and
the defects are all in the paths BETWEEN the pieces. A law per component proves the components.
It says nothing about composition, and a suite built one-law-per-verb will read as exhaustive
while covering none of the joins.

**Why it is invisible from inside:** the coverage instrument agrees with you. Nineteen laws, 100%
of the public surface, every one able to go red under mutation — every signal a test suite can
emit says "covered". The uncovered thing has no name in that vocabulary, because a seam is not a
component and nothing enumerates it.

**The tell, and it is reliable:** when a defect IS found, ask where it lived. If the answer is
repeatedly "between two things I built separately", the suite has this shape, and the next defect
will be in another seam rather than another component.

**Real incident, 2026-08-25.** `wat/gen.wat` was declared feature-complete and promoted to the
stdlib on 19 mutation-proven laws. An 18-ward vigilia the same day found: `card` and `at` able to
disagree (two fields of one struct); `one-of`/`bind` trusting an unvalidated sum of other
generators' cardinalities; a macro re-splicing a caller's expression (52x measured cost against a
comment claiming 2x); `lift2` and the `coords` path encoding the same radix twice and disagreeing;
and a law that an IDENTITY implementation passes. **Every one at a seam. Not one law crossed one.**
Two earlier gaps that session (`check` returning no witness, `shrink` composing with nothing) were
the same class and were found only because the builder asked a direct question.

**The cure is not more laws.** It is a law per JOIN: build one thing two ways and require them to
agree (this caught nothing only because the tripwire drove indices 0..5 and the divergence began
at 6); assert the SUT's own reported denominator rather than re-reading the struct; and mutate to
the do-nothing implementation, not to a wrong one — an identity passes far more gates than a
scramble does.

### Failure mode 25 — `git rm` after a failed generator, then `git checkout` eats the work

**Signature:** a script that generates a replacement file fails partway; the surrounding command
proceeds to `git rm` the originals anyway; the instinctive recovery (`git checkout -- <paths>`)
restores them from HEAD — discarding every uncommitted change made since.

**Real incident, 2026-08-25.** A python heredoc raised `ValueError: substring not found` before
writing `wat-tests/rete/differential-fuzz.wat`. The same Bash invocation's `git rm -qf` ran
regardless. `git checkout --` then restored the pre-edit version from HEAD, silently deleting an
uncommitted `bind` rewiring that had taken several steps to build. It had to be redone from the
transcript.

**The cure is ordering, not care.** Commit before restructuring — the work was green and
committable and was not committed. And a destructive step must not sit in the same command as the
step that justifies it: if generation and removal share an invocation, a failed generation still
reaches the removal.

### Failure mode 22 — A rule written in prose that nothing ever runs

**Signature:** a document states a discipline in the imperative — *"there is exactly ONE X; if
you find a second, one of them is lying — prune it"* — and no test, no script, and no step in
any checklist ever executes that check. The rule reads as enforced because it is stated
forcefully. It is a convention, and conventions rot silently.

**Why it is worse than no rule:** the rule's presence *suppresses* the audit. A reader who meets
it concludes the invariant is being maintained and moves on. The louder and more confident the
phrasing, the more effectively it prevents anyone from checking.

**The cure is extirpare's ladder, applied to the RULE and not just the code.** When you write a
rule into a document, ask which rung it is on. Prose = convention (weakest). A test that walks
the tree and fails = a check at construction. A shape where the second X cannot exist = the top
rung. If you cannot climb, say in the document *which* rung it is on, so the next reader knows
it is unenforced rather than assuming otherwise.

**Real incident, 2026-08-25:** arc 278's `SEAM.md` carried exactly that rule about breadcrumbs.
A recolligere found **four** files in the same arc each announcing itself as the one live
current-state — `SEAM.md` (pinned to a HEAD 12 days old and a floor 660 tests short),
`DESIGN-no-hidden-failures.md` (**six** stacked SEAM blocks, oldest five weeks old),
`BACKLOG.md` ("CURRENT STATE — read first", declaring the arc PARKED when it had been the active
arc for 55+ commits), and the true one. The rule was correct and had never once been run. Worse:
the recovery file's own ledger pointed at a *fifth*, wrong, file — a different arc's cliff notes
— so an instance filling the ledger honestly would read the wrong document and still tick every
box. A prior self had already recorded that exact defect as OWED in `REALIZATIONS.md:10578`; it
sat unactioned because an OWED line, too, is a rule nothing runs.

### Failure mode 23 — A deferral whose stated REASON expires, with nothing to re-read it

**Signature:** a comment or brief defers work and names a blocker — *"the macro cannot yet
marshal errors back"*, *"the checker punts here"*, *"waiting on X"*. The blocker is later fixed,
by a different strike, for a different purpose. Nothing connects the fix to the deferral, so the
deferral keeps citing a reason that is no longer true, and every reader who meets it accepts the
justification without re-testing it.

**The mechanism is the absence of a re-read.** A deferral parked in prose has no owner, no
review point, and no gate — so its premise is never revisited. This is the same shape as FM 22:
something is written down forcefully enough to stop inquiry, and nothing ever re-runs the check.

**The cure — exigere's rule, and note that BOUNDING is a real close, not a dodge:** what cannot
ship either ships now or becomes a row with an owner, a cost, and the open question stated. A row
gets re-read; a comment does not. When you bound rather than ship, say so plainly and say why —
"needs a builder's ruling on a shipped surface" is an honest close; "a later stone" is not.

**Real incident, 2026-08-25:** `src/rust_deps/cache.rs` deferred converting two `panic!` guards
to matchable values because "the dispatch macro cannot yet marshal method-internal errors back to
wat as a `RuntimeError`". Grounding it took one file read: `src/rust_deps/sqlite.rs`'s module doc
states that `#[wat_dispatch]` marshals `Result<T, E>` natively via the blanket impls, needing
**ZERO macro changes**, including `Result<Self, E>` for a constructor — which is exactly
`Lru::new`'s shape. The stated blocker had been false for some time and the deferral had gone on
citing it, in two files, tracked nowhere. What actually remained was a design question nobody had
been asked.

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
   ls ~/.claude/projects/-home-*-work-holon/memory/project_*.md
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
"Cargo is at ~/...", "If you hesitate, run `which cargo`")
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

### Failure mode 18 — Fanning N concurrent riders that each run cargo (the shared-lock thrash)

**Signature:** spawning many background riders at once, each briefed
to run `cargo build`/`cargo nextest` (e.g. its own per-dir RED gate),
against ONE workspace. They all contend on the single `target/` build
lock; each ends its turn "waiting for the background build," re-notifies
in a loop, and burns enormous tokens making no progress.

**Two compounding defects:**
1. **The shared `target/` lock serializes every build** — 16 riders
   cannot `cargo` at once; 15 block on the 1 that holds the lock.
2. **A per-rider RED gate is unwinnable by construction** — a rider's
   `cargo nextest` must compile the *whole workspace*, which can't go
   green while *any other* rider has a half-migrated file. No single
   rider controls the state its own gate depends on.

**Reality check:** if a fan-out has each worker compiling/testing the
same workspace, STOP. That is not parallelism — it is N-way lock
contention plus N unwinnable gates.

**The fix (proven, arc 278 crusade, 2026-07-18):** **riders do TEXT
edits only — forbid cargo in the brief** ("⛔ TEXT CHANGES ONLY. Do
NOT run cargo/build/test. The orchestrator measures centrally."). The
**orchestrator weighs CENTRALLY, once**, after the tree is quiescent —
the build is a serial resource, so serialize it deliberately at the one
place that owns integration. Weigh per-dir with `binary_id(wat::<dir>)`
(a clean per-dir Summary), fix cascades, then commit.

**Real incident, 2026-07-18:** the first crusade fleet — 16 riders,
each with a `cargo nextest … RED gate` — deadlocked in build-wait
loops (riders re-notifying 4×, ~150K tokens each, ~1M ms, zero
completions). Killed the fleet; re-launched **edit-only** riders (no
cargo); they finished clean and fast, and the orchestrator ran the
one central weigh. Builder direction: *"let's just do the text changes
and measure the tests afterwards… the riders can handle the text
changes."*

**The four questions on this discipline:**
- Simple? — "riders edit; orchestrator measures once" is one serial
  build, not N contending ones.
- Honest? — a per-rider gate that can't pass (workspace it doesn't
  control) is a gate that lies; central weigh is the honest measure.

**Cross-reference:** FM 12 (model explicit), Section 7 (delegation).
Distinct from both: this is about *concurrency of the build itself*.

### Failure mode 19 — The rider believes it is the ORCHESTRATOR (the yielded background job)

**Signature:** a spawned rider launches a long verification (`cargo nextest`, a build, a grid run)
in the BACKGROUND, then ends its turn saying something like *"I'll wait here without further
polling — the background run will notify me automatically."* The harness fires a
`task-notification` marked **completed**. The rider has done the work and reported **nothing**. Its
job may still be running.

**Reality check:** a rider has `run_in_background` and a Bash tool that look identical to the
orchestrator's, so it reasons the way the orchestrator correctly reasons — *start the long thing,
end the turn, get woken.* **But ending a rider's turn TERMINATES it and returns control upward.**
Nothing wakes it. The affordance is present; the lifecycle semantics are not. That is a trap by
construction, not carelessness — which is why "be careful" does not fix it and why it recurs.

Note the failure looks *disciplined*: "don't poll, you'll be notified" is correct orchestrator
practice. The rider applied a good rule from the wrong tier.

**Real incident, 2026-08-01 (arc 278, the compiled-conditions stone):** the rider finished the
implementation, launched `cargo nextest run --release` backgrounded, and yielded. The notification
read "completed" with a result that was one sentence about waiting. `pgrep` showed its nextest still
alive at load 16.8, and its work sitting uncommitted in the tree. It was resumed via `SendMessage`
and delivered a complete, high-quality report — nothing was lost, but the orchestrator had to notice
a "completed" agent that had not reported, rather than reading a report.

**The prescription — brief the ROLE, not just the rule.** A bare prohibition gets reasoned around by
a capable model; the role makes the rule derivable:

> You are a rider, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you, and
> nothing will wake you. There is no notification coming. Run every verification in the FOREGROUND
> and block on it: your turn ends when the numbers are in your hands, not when the command is
> launched.

**Orchestrator-side, when it happens anyway:** do NOT run cargo to check on it — a second build
against the same `target/` lock is FM 18, and any number taken while the rider's job is live is an
instrument artifact. `pgrep` for its process, confirm the work in `git status`, and `SendMessage` the
agent to finish and report. A resumed rider keeps its full context.

**Related but distinct:** FM 18 is N riders contending on one build. This is ONE rider whose turn
ended while its own build ran. Both are about how rider work relates to the build; neither is about
the rider's competence.

### ⛔ AMENDED 2026-08-18 — THE ROLE-BRIEFING PRESCRIPTION ABOVE DOES NOT WORK. FM 18 ALREADY HELD THE FIX.

The prescription above says to brief the ROLE ("ending your turn ENDS you"), on the theory that a
bare prohibition gets reasoned around but a derivable rule sticks. **Measured across one day, arc 118
route B: FOUR riders, FOUR occurrences.** Every brief carried the role paragraph verbatim. The fourth
also carried the running count and the sentence *"if you catch yourself about to background a build
or a floor: don't."* It backgrounded the floor and ended its turn anyway.

Four for four with the prescription applied is not a briefing problem. **The affordance is the
defect**: a rider holds a Bash tool with `run_in_background` and a long verification to run, and the
orchestrator's own correct pattern — start the long thing, end the turn, get woken — is the obvious
move from inside that position. Telling a capable model not to take the obvious move, in prose, is
the CONVENTION rung.

**The fix is one rung up, and FM 18 already wrote it down:**

> *riders do TEXT edits only — forbid cargo in the brief. The **orchestrator weighs CENTRALLY,
> once**, after the tree is quiescent.*

FM 18 derived that from lock contention among N riders, so it reads as a concurrency rule and I
applied it only to fan-outs. **It is not a concurrency rule. It is a tier rule, and it eliminates
FM 19 by construction:** a rider that never runs the floor cannot background one. The single-rider
case felt exempt because there is no contention — but contention was never what made FM 19 fire.

**The standing pattern, both modes, one line:** the rider edits and reports; the orchestrator builds,
floors, and clippies. Give the rider the acceptance CRITERIA so it knows what "done" means and can
run cheap targeted checks (a `--check`, a single probe, a scoped `nextest -E`); do not give it the
floor. The orchestrator was independently re-running every load-bearing row anyway — examinare's
*weigh the kill against your own reading of the disk* — so handing the rider the full floor bought
a duplicated 4-minute run and a 50% chance of losing the report.

**What the rider still owns:** its own numbers for anything the orchestrator cannot reconstruct —
which sites it inspected, what a perturbation did, what surprised it. Those are the honest deltas,
and they are the reason to resume a rider rather than take over.

★ **The meta-lesson, and it is the expensive one:** FM 19's prescription was written from ONE
incident and never re-measured. It read as sound and was cited in three subsequent briefs while
failing in each. A prescription is a claim; four occurrences are its refutation.
`[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`

---

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
- [ ] **For non-trivial substrate compositions named in the BRIEF (e.g.,
      "use splice + Vector/map + runtime quasiquote"), you have written
      a `tests/probe_diagnostic_<topic>.rs` that proves the composition
      empirically. The probe is committed; the BRIEF cites it verbatim
      as "the working pattern sonnet must mirror." Per FM 2-bis —
      grep is insufficient for composition claims; the empirical probe
      is the orchestrator's earned right to assert the composition.**
- [ ] Where the substrate doesn't support what the brief asks, you have
      EITHER (a) added a prior slice that fixes the substrate, OR
      (b) explicitly scoped the brief to not depend on the missing piece
- [ ] **No STOP-trigger in the BRIEF reads as a permission-to-defer slot.**
      Per FM 2-bis — STOP triggers are REJECTION criteria. Signal-phrase
      audit: search the BRIEF for "STOP-X (substrate lacks Y): surface
      as finding" / "if Z cannot be expressed cleanly" / "if this
      approach doesn't work, fall back to" — each is an orchestrator
      pre-drafting the deferral path. Rewrite as hard rejection or
      replace the BRIEF with a substrate-extension stone request.
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
- `~/work/holon/CLAUDE.md` — workspace setup (auto-loaded)
- `~/work/holon/wat-rs/CLAUDE.md` — wat-rs guidance (if present)

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

### Arc-specific cliff notes (load-first when present)
Some arcs grow oversized realization docs that blow context if loaded in full.
When that happens, a `CLIFFNOTES.md` sibling lives next to the full doc — load
that FIRST; the full file is for date-indexed deep-reads only.

Currently:
- `wat-rs/docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-CLIFFNOTES.md`
  — compresses 6722-line `INTERSTITIAL-REALIZATIONS.md` to ~150 lines / ~5K tokens.
  Preserves: 15-floor trajectory, doctrines table, 13 convergences (pointer to
  `project_convergences` memory), 15-song operational soundtrack, recurring
  mistake patterns, hologram/strange-loop framing, current-state breadcrumb.

**Pattern**: when a realizations/interstitial doc exceeds ~1000 lines, inscribe
a `CLIFFNOTES.md` sibling that distills load-bearing doctrines + convergences +
recurring patterns + current state. Both stay; cliff notes can be refactored (it
IS an index); the full file is immutable historical record per
`feedback_inscription_immutable`. New realizations inscribe in the full file
first; then the cliff notes' "Currently..." section + load-bearing distillation
get updated.

### Closure-discipline tracker
- `wat-rs/docs/arc/2026/04/109-kill-std/DEFERRAL-VIOLATIONS.md`
  — running tracker of arcs marked INSCRIBED while carrying
  open deferrals. The 2026-05-03 audit identified violations
  across pre-109, post-109, and same-session-as-doctrine arcs
  (incl. arcs I closed myself). Per FM 11 + Section 11's
  pre-INSCRIPTION grep, this tracker should shrink, not grow,
  going forward. New violations land here when caught.

### Memory (already auto-loaded)
- `~/.claude/projects/-home-*-work-holon/memory/MEMORY.md`
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
# WRAP-PROOF form (2026-06-06): a line-based grep is BLIND to phrases broken
# across wrapped lines ("If/when\n  a caller surfaces" — real false-pass caught
# at the arc-249 INSCRIPTION). Normalize whitespace FIRST, then match.
# CASE-INSENSITIVE (2026-08-19, arc 118's INSCRIPTION): the pattern was `-oE`, so
# `out of [a-z...]*scope` could only match LOWERCASE — and the affirmative form is a
# SENTENCE OPENER ("Out of arc N's scope. Tracked in ..."), so the one phrase the
# gate most needs to surface for judgement was the one case it could not see.
# Under-reported the acceptable form, and would also have slipped "Out of scope;
# we'll get to it" — a real false-pass path. `-oiE` now:
tr '\n' ' ' < <INSCRIPTION> | tr -s ' ' | grep -oiE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of [a-z0-9' ]*scope|when a caller[a-z ]*|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" | sort | uniq -c
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

---

## Section 13 — IPC contract for wat processes (stdout / stderr / exit-code triangle)

**Canonical model for how wat processes communicate complex return values + error values back to their parent.** Established arc 170 REALIZATIONS pass 10; load-bearing for spawn-process, run-hermetic, wat-cli, and any future IPC primitive.

### The triangle

| Channel | Carries | When read |
|---|---|---|
| **stdout** | Complex return values (multi-line / structured / EDN-serialized) | Exit code 0 (clean exit) |
| **stderr** | Complex error values (panic cascades; explicit eprintln EDN) | Exit code non-zero (panicked) |
| **exit code** | Binary signal: 0 = clean / non-zero = panicked | Always; tells parent which channel holds the result |

The exit code is JUST A SIGNAL — it discriminates which channel the parent should consume. It does NOT carry the value itself. Complex values live in stdout / stderr (where there's room for structured data).

### Why no ExitCode return type

Earlier in arc 170 (slice 1c / 2), a `:wat::kernel::ExitCode = :wat::core::u8` typealias was minted, and `:user::main`'s signature was `[] -> :wat::kernel::ExitCode`. **REALIZATIONS pass 10 retired it** (implemented in slice 1e).

The rationale (per `docs/arc/2026/05/170-program-entry-points/DESIGN.md` § "Nil IS the exit code (no ExitCode type — superseded 2026-05-10)"):

- An ExitCode return type CONFLATES "exit signal" with "return value." stdout already carries return values; the exit code only needs to signal which channel holds the result.
- The substrate already maps panic-cascade to `libc::exit(N)` via slice 1i's StdErrService epilogue (the panic chain has the exit semantic baked in).
- Clean nil-return maps to `libc::exit(0)`. No information lost; semantics preserved.
- `:user::main`'s canonical signature is `[] -> :wat::core::nil`. Uniform with arc 114's `Program<I,O> -> :nil` shape across tier 0 (thread) / tier 1 (process) / tier 2 (remote).
- Scope-out: *"future arc may mint a helper if a CLI tool genuinely needs 0/1/2 exit-code distinction; arc 170 affirmatively does not."*

### For program authors

- **`:user::main -> :wat::core::nil`** — return nil for clean exit; do NOT try to return a value
- **Complex return values** — write to stdout via `:wat::kernel::println` (auto-EDN-encodes via the trio's StdOutService)
- **Complex error values** — let panics propagate (substrate cascades structured `#wat.kernel/ProcessPanics` EDN to stderr) OR explicit `:wat::kernel::eprintln` of your own structured EDN error data
- **Reading the result** — parent inspects exit code first; reads stdout (success) OR stderr (panic) accordingly; either channel may carry multi-line EDN

### For substrate / macro authors

- spawn-process, run-hermetic, run-thread, wat-cli, run-hermetic-with-io — ALL preserve this triangle
- Any future IPC primitive must respect it; do NOT mint an "ExitCode" or "ReturnValue" type that competes with stdout
- Do NOT add a fourth channel for "values"; stdout IS the values channel
- Do NOT add a fourth channel for "structured errors"; stderr IS the structured-error channel (panic cascades use it canonically)

### STOP signal

If you reach for a shape like `:user::main -> :SomeReturnValueType` or a `spawn-process` variant that returns "the value" separately from stdout — STOP. Re-read this section. The triangle is sufficient. Adding shapes to it competes with the canonical model + creates "two ways to do the same thing" — anti-pattern per `feedback_wat_llm_first_design`.

### User direction 2026-05-15

> *"users communicate 'exit values' via stdout and there's only panic + value for stderr — (stdout, 0) and (stderr, 1)"*
>
> *"this is how we manage complex return values - we just write to stdout. if the exit code isn't 0 then the value is on stderr - stderr communicates complex error values and stdout communicates complex return values"*

Cross-references:
- `docs/arc/2026/05/170-program-entry-points/DESIGN.md` § "Nil IS the exit code"
- `src/stdlib.rs:127` (the retirement note pointing to REALIZATIONS pass 10)
- FM 7-ter (thread context illegality) — same axis at a different layer: threads share parent's stdio; processes get their own captured stdio via the trio

