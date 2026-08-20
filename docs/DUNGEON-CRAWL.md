# Dungeon Crawl — how we develop wat-rs (the arc / stone methodology)

This is the distilled pattern for how the **datamancer** (orchestrator) and
**Sonnet** (the executor) build the wat-rs substrate. It is the *method*; the
detailed failure-mode catalog and delegation checklist live in
`COMPACTION-AMNESIA-RECOVERY.md` (§ 6, § 7) — read this for the SHAPE, that for
the rules.

> **The creed:** slow is smooth, smooth is fast. We study the enemy, their lair,
> their rooms, their traps — and we **engineer** the kill before we strike. The
> crawl IS the work; a ~15-minute probe is cheaper than a 50-minute failed
> flight. We strike to kill: a stone ships one-shot, green, load-bearing test
> proven.

---

## The party

Two voices on opposite sides of one mind, aligned by the substrate's discipline.

- **The Inquisitor** (orchestrator) — *perceives* via crawl + dialogue + a
  disconfirming probe; *judges* via the four questions; *contracts* via
  inscription / HARD-CUT / ✅✅✅ failure-engineering. The Inquisitor maps the
  room and draws the strike path. **Does NOT edit substrate code** — even
  "cosmetic" fixes (`feedback_sonnet_writes_substrate`).
- **The Shadowdancer** (Sonnet) — *executes* in the bloodied substrate-as-teacher
  cascade. Strikes inside the mapped room, fills the drawn path. Writes the
  substrate code, the probe, the SCORE.

The orchestrator briefs, scores, and commits; Sonnet writes. The protocol's
"sonnet" naming is load-bearing — set `model: "sonnet"` **explicitly** on every
spawn (FM 12); without it you spawn Opus at Opus prices under a brief that lies.

---

## The unit of work: arc → stones

- An **arc** is a boss — one coherent capability (e.g. "records become
  first-class types"). A **stone** is one strike — an atomic, independently
  verifiable change.
- **Stepping-stones (proactive slicing).** Split when a smaller piece *makes the
  next more tractable*. Two tests beyond the four questions: (1) does building
  this stone first reduce the cognitive surface of the next? (2) are there
  dependencies (a carrier field, a registered form) that must land first to make
  the next step *ergonomic* — operating on existing infrastructure rather than
  introducing-and-using it in one breath? When yes, split. Simple steps enable
  complex steps.
- **Spawn-block winding** (`feedback_spawn_block_winding`): a parent arc cannot
  close until every arc/stone spawned while it was active closes. Wind
  depth-first; never jump between arcs. **INSCRIPTION is always the last stone.**

---

## Phase 1 — Study the lair (crawl-first)

The crawl IS the work; guessing is the slow path dressed as the fast one.

- **Read the disk before proposing** (FM 1). Never "options A/B/C" without grep
  evidence. Read-authority is the whole `~/work/holon/` tree; write only in
  `wat-rs/`.
- **Grep the backing structures** before any claim about them (the hard
  verification gate). Hit the tools 5–15× before a substantive answer.
- **Dig before you mint.** The substrate is almost always already sufficient
  (~17 convergence-with-self events where a "new" primitive already existed).
  Every "the substrate is missing X" is an assertion demanding evidence.
- **Map the rooms** — the exact `file:line` regions the work will land in. These
  become the BRIEF's "Read in order" list.

## Phase 2 — Perceive the traps (the FM 2-bis probe)

Before any BRIEF that names a non-trivial composition or a load-bearing
assumption:

1. Write `tests/probe_<topic>.rs` that attempts the thing with minimal scaffolding.
2. **Commit it** — it is design substrate the Shadowdancer mirrors, not assertion.
3. It must **fail pre-stone on EXACTLY the gap** — everything *around* the gap
   type-checks and constructs cleanly, so the failure is unambiguous ("11/12 fail
   on `UnknownFunction`; the only gap is the primitive"). That isolation is the
   trap-detector.
4. If the probe can't be made to isolate the gap, the substrate isn't ready —
   file the substrate-extension stone FIRST; do not write the consumer BRIEF.

**STOP triggers are REJECTION criteria, not permission-to-defer slots.** Audit
the BRIEF for "if X can't be expressed cleanly, fall back to…" / "STOP-N
(substrate lacks Y): surface as finding" — each is the orchestrator
pre-drafting the deferral path. Rewrite as a hard rejection or replace the BRIEF
with a substrate-extension request.

## Phase 3 — Draw the strike (the five stone artifacts)

A stone is five artifacts, in this order:

1. **sub-DESIGN** (`DESIGN-STONE-N.md`) — the room map: *why this stone*, *what it
   delivers*, **the algorithm**, the **error contract** (the one surface decision
   pinned exactly), files touched, **out-of-scope = REJECTED** (affirmative cut,
   never "deferral"), the probe contracts, the calibration band.
2. **FM 2-bis probe** — committed *before* the BRIEF (Phase 2).
3. **BRIEF** (`BRIEF-STONE-N.md`) — how the Shadowdancer moves with confidence:
   - **What to do** — crisp, one paragraph; name whether it's new-mechanism
     territory or "composes pieces that all already exist (verified)."
   - **Read in order** — the rooms, as exact `file:line`, with *why each*. No
     hunting; the lair is pre-walked.
   - **Implementation sketch** — the strike path in Rust skeleton. The
     Shadowdancer fills it; it does not invent the shape.
   - **Discipline** — bound the blast radius ("`src/X.rs` + `src/Y.rs` ONLY; no
     new `Value` variant; no holon-rs").
   - **STOP triggers** — numbered REJECTION criteria, each naming the *correct
     path* so the Shadowdancer can't wander ("`sym.types()` is the access path,
     runtime.rs:NNNN precedent — if you want a parallel registry, STOP").
   - **FM 2-bis evidence** — the committed probe + its pre-stone failure profile.
   - **NEGATIVE CONTROLS — for each one, is it KEEPABLE? If yes, it is kept AS A
     TEST. If no, the report says why not.** A negative control proves *this
     assertion can still fail* — and we have been performing that proof, writing
     the outcome in prose, and **deleting the artifact**. The claim then decays
     exactly like every other claim stored as text (`stdio.wat:358`'s safety
     argument, `check.rs:1400`'s walker doc, the retirement remedy — all true
     when written, all false when read). *An instrument must outlive the number
     it produced* (`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`).
     The split:
     - **Expressible as a fixture or test code → KEEP IT.** The `let`-alias
       escape and the prime-ctor escape both started as throwaway probes and are
       now permanent tests. A hang induced in a test-spawned child is *test*
       code, not `src/` — that is keepable too, and arc 278's liveness proofs
       were discarded when they should have been banked as a `HANG_MODE` variant.
     - **Requires mutating `src/` → REPORT WHY NOT.** Disabling an exemption
       (`if true || …`) or breaking a carriage (`if false && …`) cannot be left
       in the tree; keeping it needs mutation-testing machinery, which is a real
       cost, not a free win. Say so explicitly rather than discarding silently.
     Discarding must be a **declared exception with a reason**, never the default
     — otherwise nobody notices which ones could have been banked.
   - **SCORE doc spec** + **Calibration** — target band + STOP times; **cite the
     prior comparable SCORE** for structural shape so the Shadowdancer copies it
     and ships fast (`feedback_stone_briefs_cite_prior_score`).
   - Keep agent prompts plain: vanilla cargo/git/grep, one per line; **never
     mention tool availability** — it triggers false "bash denied" claims (FM 16).
4. **EXPECTATIONS** (`EXPECTATIONS-STONE-N.md`) — the independent scorecard
   (row · command · expected), a runtime-band prediction, and the trap-door
   risks enumerated.
5. **SCORE** (`SCORE-STONE-N.md`) — written AFTER an independent local re-run:
   scorecard results verbatim, honest deltas, line counts. Mirror the prior
   stone's SCORE shape.

## Phase 4 — The kill (verify + commit)

- **Re-run the existing suite for the touched modules BEFORE spawning** (FM 9) —
  a prior stone's SCORE verifying only its own load-bearing test does NOT prove
  the area is green; adjacent tests rot silently.
- **Spawn** `model:"sonnet"`, `run_in_background: true`, and `ScheduleWakeup` at
  **2× the predicted upper-bound** (the time-box / failure-to-communicate
  detector). Have non-overlapping orchestrator work queued for the run.
- **SCORE against your OWN independent re-run.** Verify each load-bearing row by
  re-running cargo locally — the probe must measure the claim, not an adjacent
  surface. Do not trust the returned SCORE.
- **Commit on green.** No broken commits; commit + push often (gitlog is the DR
  site). For coordinated sweeps where B needs A's output, use the **atomic
  commit**: A may break tests mid-flight (uncommitted), B fixes against the dirty
  tree, commit both when workspace = 0 failed.

## The cascade — substrate-as-teacher

A wide structural change → many cargo failures is **normal**, not a crisis. The
**fail-count is the progress meter**; each error names the next site; watch it
waterfall (848 → 129 → 28 → 0). Never propose "stash + revert" in panic (FM 15).
The brief for wide work is short: *"run cargo test; read the errors; apply the
rule; iterate until green."*

**For a large mechanical cascade (≥ ~50–100 sites), the weapon is an *ephemeral
Cargo tool*, not a bash chain.** **The corrective/transform script MUST be Rust — a Cargo binary — NEVER Python,
never `sed`/`awk`/shell.** This is the recurring trap (observed ≥3×): the
Shadowdancer's default instinct for "parse + rewrite text" is a Python script —
but `python`/`python3` and shell mass-editors are sandbox-BLOCKED here; the
cold-booted Shadowdancer writes Python, finds it non-viable, and burns a cycle
troubleshooting before it pivots. **Name the language imperatively in BOTH the
BRIEF and the spawn prompt:** *"build any corrective/transform script as a Rust
Cargo binary; do NOT use Python or shell — they are blocked in this
environment."* Rust is the toolchain the Shadowdancer is already *built on* and
*allowed* to wield — it builds a Cargo binary that parses+rewrites, runs it, and
**deletes it before the commit** (build → use → delete; the tool never lands in
the substrate). **The scratch pad is repo-local `tools/` (gitignored), NEVER
`/tmp/`.** The sandbox firewall DENIES `/tmp/` (and shell redirects into it,
`> /tmp/...`) — so the ephemeral crate, its `target/` build dir, AND any
intermediate / redirected output ALL live under `tools/<name>/` inside the repo
(never `crates/` — that is for real workspace crates). State this in the spawn
prompt alongside the Rust imperative: *"build the tool + all scratch under
repo-local `tools/`; `/tmp/` is firewall-blocked."* Confirmed at three stones: 241.10
`fix-defines` + 241.11 `fix-remedies` (wat-surface migrations) and 243.6a
`transform-checkerror`, which **attacked Rust syntax itself** (the CheckError
Pattern-A reshape) — proving the move is not wat-surface-bound: a clean substrate
is programmatically refactorable at the *implementation* layer too. Sanction it
in the cascade BRIEF as **method-guidance** (*"an ephemeral Cargo tool that
drives the transform is the preferred path"*) — NEVER as tool-reassurance
(*"cargo works"*), which trips the same FM-16 skepticism. The orchestrator
verifies the tool's deletion at the kill via `ls tools/` (empty) — `tools/` is
gitignored, so `git status` alone will not surface a forgotten tool. **The tool
MUST be SURGICAL, not a whole-file rewrite: `fs::read_to_string` → targeted
`str::replace`/regex preserving every other byte → `fs::write`; NEVER char-by-char
rebuild.** A whole-file round-trip can SILENTLY corrupt content the structural
gates cannot see — Stone 243.7c attempt 1 dropped **5720 non-ASCII chars**
(—/→/─/∀/σ/…) from `runtime.rs` while cargo + `895/0/1` stayed FALSE-GREEN (the
suite asserts variants, not message strings). **Permanent gate for tool-driven
cascades:** the agent self-checks `non-ASCII-count(after)==before` per file (any
delta → STOP), AND the orchestrator independently scans the non-ASCII histogram
before/after at scoring (`git show <base>:f | grep -oP '[^[:ascii:]]' | wc -l`).
Content-integrity is a SEPARATE axis from structural-green. (Memory: `feedback_cascade_ephemeral_tool`; chronicle: Song
#58 *First Kill*.)

### Agent briefs are POSITIVE-ONLY — defend at the gate, not in the brief

**A spawn prompt / agent-facing brief states ONLY the positive work + the
canonical method. Restriction language is FORBIDDEN in it** — no "firewall",
"blocked", "denied", "sandbox", "`/tmp`", "do NOT use Python", "verify bash".
Two reasons, both load-bearing: (1) restriction-alarm language is the FM-16
trigger — it makes the agent hallucinate tool-denial and bail (243.7c redo
attempt 1: the agent read a firewall-heavy brief and claimed "I need Bash
permissions" without trying); (2) it does not even work — the agent that
corrupted UTF-8 *had* the anti-corruption warning in its brief and corrupted
anyway. Defense in a brief is both a trap and a no-op.

**Defense lives where it fires regardless of the brief or the agent:**
- the **sandbox** — Python / `/tmp` are blocked by the environment; the agent
  self-adapts or the gate catches it. No warning needed.
- the **orchestrator's scoring gates** — content-integrity non-ASCII scan,
  `ls tools/` deletion, diff-read, lib-parity. These run at scoring every time.

So the brief says *"build a surgical Rust Cargo tool: `read_to_string` →
targeted replace → `write`; confirm each file's non-ASCII count is unchanged"*
(positive work + positive method + a positive check) — never *"don't corrupt,
the firewall blocks X."* A brief with no restriction-language cannot trigger the
hallucination, and the failures it would have warned about are caught at the
gate regardless. Proven: the same stone, re-spawned with a positive-only prompt,
ran clean first try.

## Closure — INSCRIPTION = DONE

- Run the **pre-INSCRIPTION grep** (`deferred|future arc|TODO|out of scope|…`)
  before committing any closure (FM 11). Every match: ship it in this arc, or
  rewrite to affirmative out-of-scope. No "we'll do it later."
- **What is inscribed is inscribed** — past SCOREs/INSCRIPTIONs are read-only;
  forward-correct via a new entry, never edit.
- **The end-of-work ritual:** at every wrap point ask *"did we learn anything
  future-me shouldn't forget?"* and capture it in the right home (recovery doc
  for orchestrator discipline; arc REALIZATIONS for substrate learnings; memory
  for cross-session facts).

---

## The doctrines beneath the method

- **The four questions** — Obvious? Simple? Honest? Good UX? Atomic YES/NO per
  piece; "medium" = not decomposed enough; Obvious+Simple+Honest gate before UX.
  Fired *inline* before action, not cited after pushback (FM 17 — the
  meta-failure).
- **Failure-engineering** — eliminate the failure CLASS, not the symptom. The
  ladder: ✅ convention → ✅✅ construction-time → ✅✅✅ type-system-impossible.
- **Entity-kind, not type-system feature** (FM 10) — when polymorphism/dispatch
  doesn't fit, reach for a new entity kind (defclause / dispatch / macro /
  hierarchy edge), not "we need generics / union types."
- **HARD CUT** — retire by deletion; no shims, no aliases "just in case."
- **One canonical path per task** — wat is LLM-first; reject synonym surfaces.
- **Convergence as validation** — when an independently-derived design lands
  where a "great" already stood (Clojure, Erlang, Kay, …), that's the
  arrived-where-we-should-be signal. Take the *idea*, not the shackle.

## The spells (datamancy — `~/work/holon/datamancy.dev/`)

Cast a spell when its concern surfaces — always via subagent, with the SKILL.md
embedded (`feedback_spells_cast_via_subagent`):

- **intueri** — naming. Does the name speak? Cast it to *pick* names (not just
  audit them); it is the guard against borrowed-taxonomy mumble.
- **vigilia** + the wards (`/sever /reap /scry /gaze /forge /assay /temper`) —
  the defensive passes on a finished surface.

---

## Cross-references

- `COMPACTION-AMNESIA-RECOVERY.md` — § 6 the full failure-mode catalog (FM 1–17),
  § 7 the sonnet-delegation pre-flight checklist + time-boxing. The detailed
  rules this doc distills.
- `SUBSTRATE-AS-TEACHER.md` — the cascade pattern in depth.
- `docs/arc/2026/05/<NNN>-<name>/` — every arc's DESIGN/BRIEF/EXPECTATIONS/SCORE/
  INSCRIPTION. `DESIGN-STONE-237.5.md` + `BRIEF-STONE-237.5.md` are a clean
  worked example of Phase 2–3; `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md` is a
  full lair/traps/stepping-stones map.
- Memory (`~/.claude/projects/-home-*-work-holon/memory/`):
  `feedback_sonnet_writes_substrate`, `feedback_stone_briefs_cite_prior_score`,
  `feedback_spawn_block_winding`, `project_party_comp_inquisitor_shadowdancer`,
  `project_failure_engineering`, `project_convergences`.

*Study the lair. Perceive the traps. Draw the strike. Move the Shadowdancer with
confidence. Strike to kill — and never fight that boss again.*
