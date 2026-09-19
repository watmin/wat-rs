# REPLAY PLAYBOOK — how to land a large divergent branch, one commit at a time

**Distilled 2026-09-19 from the grok-rete replay: 651 commits, ~7 months of source history, landed in
27 batches over ~2 weeks of wall clock. Floor green at every checkpoint. Zero source content dropped.**

This is the method, not a history. The history is `the-grok-rete-replay/` — 58 briefs, 48 scores, a
4,687-line log, and `FINDINGS-composition.md`'s 40 findings. **Read this file first; go there for a
precedent when something here says "there is a precedent".**

---

## 1. When this applies

Use a replay when **both** are true:

- the branch is **long** (hundreds of commits) and **diverged** (main moved underneath it), and
- **main owns something the branch also touches**, so a whole-tree merge would silently pick a winner.

The alternative — one big merge — was tried here first and **rejected**: it dropped main's hand-written
content in 34 files, and nothing failed. `merge/grok-rete` is kept as a crib and a cross-check. **A
merge that compiles is not a merge that is correct.**

⛔ **The ruling that makes this tractable: MAIN OWNS SYNTAX, THE BRANCH OWNS ITS SUBSYSTEM.** Write that
boundary down before step 1. Every collision later is adjudicated against it.

---

## 2. The invariants — break these and the rest is theatre

1. **One commit at a time, in the branch's own order.** Each lands as
   `REPLAY(<branch> #N): <the source commit's own subject, verbatim>` with a
   `(cherry picked from commit <sha>)` trailer.
2. **Never a knowingly-red commit.** If a step cannot land green, it is a STOP, not a TODO.
3. **The fold rule.** A composition defect belonging to step N folds **into N**, and N+1… are rebuilt.
   Never a repair commit after the batch. ⚠ Exception (the #184 precedent): a gate that *lands* at step
   N is repaired **at N** — it did not exist earlier, so earlier steps were not wrong.
4. **Never rewrite published history.** Prove it every batch: `git merge-base --is-ancestor origin/<branch> HEAD`
   plus an empty `refs/original/`. ⛔ **Never `git filter-branch`** — detach, re-commit, rebuild
   descendants.
5. **There is no known flake.** A red is a red. Capture it whole the *first* time; a re-run that goes
   green destroys the only evidence.
6. **The subject and the trailer are COPIED, never typed.** `SUBJ=$(git log -1 --format=%s <C>)`,
   `SHA=$(git rev-parse <C>)`, piped into the message. Six trailers were fabricated across this campaign,
   every one self-caught, every one costing a rebuild.

---

## 3. Setup — build the plan before landing anything

Produce, once, under `bootstrap/era/replay-plan/`:

| table | what it answers | why it earns its keep |
|---|---|---|
| `commits.tsv` | `N → source SHA`, plus per-step file/wat/rs counts | the spine; every census and gate reads it |
| `flags.tsv` | per step: docs / code / shared, codemod touches | cheap batch shaping |
| `absent-on-main.tsv` | paths the branch edits that **main renamed or deleted** | the hazard rows — each is a landing decision |
| `stdlib-touch.tsv` | steps touching the stdlib | flags the expensive ones early |
| `main-changed.txt` / `main-deleted.txt` | main's own movement under the branch | what the whole-merge got wrong |

⛔ **Establish the per-step RECORD FORMAT at step 1, and gate it from step 1.** Ours was invented around
step **153**. The record gate therefore cannot verify steps 1–151 and never will — re-litigating 150
closed batches against a convention that postdates them was correctly refused, but it left a permanent
hole in an otherwise complete audit. **This is the single cheapest thing to get right and we got it
wrong.**

---

## 4. The loop — one batch, start to finish

**Batch size: 20 steps.** Big enough to amortise the ceremony, small enough that a fold rebuilds in
minutes rather than hours. The final batch was 11; that was fine.

### 4.1 Census (orchestrator, ~15 min)

Derive **from the data, over the whole range**:
- docs-only vs code steps; the file-extension histogram;
- hazard rows, `wat/` paths, new gate files, codemod edits;
- the **test delta**, and every new `.wat` run through the checker **before** release;
- divergence per step: how many touched files already differ from the source's pre-image here.

⛔ **Never hand-list the files to inspect.** Derive the set. My hand-lists were short three batches
running.

### 4.2 Brief + Expectations (orchestrator, ~30 min)

Two documents, committed **before** the work starts, then pushed:

- **BRIEF-<n>** — the trap doors, each with the measurement behind it, and what to do at each.
- **EXPECTATIONS-<n>** — numbered rows the orchestrator will re-run. ⛔ **State plainly which rows were
  PRE-FLIGHTED and which are predictions**, and **what you could not measure and did not pretend to**.

⛔ **A prediction is a prediction.** Say in the brief that disproving one is a RESULT. Say **"if this
brief contradicts the source's diff or the runner, they win — report it."** That clause caught four of
the orchestrator's own wrong numbers in this campaign.

### 4.3 Release an executor (a subagent, not the orchestrator)

**Why the split is load-bearing, not bureaucracy:**
- the executor runs *filtered* gates only; **the whole floor and clippy are the orchestrator's row**, run
  uncontended, so a green is never bought by a contended machine;
- the executor cannot push, so nothing reaches the remote unverified;
- the orchestrator re-derives every claim independently. **Executors corrected the orchestrator's floor
  prediction four times and were right every time.**

Give the executor: the brief, the expectations, the doctrine file (it is **not** auto-injected to
subagents), the findings, and the previous batch's score.

### 4.4 Land, verify, checkpoint (orchestrator)

On hand-back, **re-derive rather than accept**:

    scripts/replay/verify-step-record.sh <batch-start> HEAD <first> <last>   # exit 0
    # per step: subject == "REPLAY(<branch> #N): $(git log -1 --format=%s <C>)"
    # per step: trailer SHA vs commits.tsv, two-sided
    git replace -l; git for-each-ref refs/original/                          # both empty
    git merge-base --is-ancestor origin/<branch> HEAD                        # ancestor

Then the checkpoint, **uncontended**, with the test-count prediction **locked in a file first**:

    scripts/floor.sh                                    # read the Summary line, never a piped exit code
    cargo clippy --release --all-targets -- -D warnings
    scripts/replay/census.sh --diff <prev> <curr>       # no STOP-8

**Green → push. Red → fold (§2.3), rebuild, re-verify inert, re-run.** Prove a rebuild inert by
comparing each step's own delta before and after — `git diff` with **`index` AND `@@` stripped**, because
a hunk header moves whenever an earlier step adds a line.

### 4.5 Close

Update the breadcrumb's **ledger row** (not just the header — see §7), the batch score, and the log.

---

## 5. The tooling

| tool | invocation | what it is for |
|---|---|---|
| record gate | `scripts/replay/verify-step-record.sh <from> [<to>] [<first> <last>]` | every step present once, sources match, and **each wall's verdict line is in the commit body** — derived from the step's own diff |
| check census | `scripts/replay/census.sh` → `.census/<utc>.txt`; `--diff PREV CURR [produced.txt]` | whole-tree `wat --check`; a file going rc 0 → non-zero that the step did not produce is **STOP-8** |
| conversion chain | `scripts/replay/convert.sh <rev> <out-dir> <path>…` | replays a source-era file through main's **recorded migrations**; the tree is never written |
| chain order | `scripts/replay/chain-order.sh <base> <main>` | derives the migration chain in landing order |
| the floor | `scripts/floor.sh` | runs the release floor **and captures it** before anyone reads it |

⛔ **The census gate is the one that catches what tests cannot.** A source-branch cure once narrowed
`--check` so 1052 of 2165 tracked `.wat` began failing — **the entire test suite stayed green**, because
the change lived in the CLI path and the floor never shells out. Only the census saw it.

---

## 6. Pre-flight checklist — every item here bit more than once

Run this at census time, before writing the brief:

- [ ] **New gate in the range?** Measure our blast radius **before** release. Three of the last six
      new gates landed red here. Read the gate's **own constants** (`SUBJECTS`, `ROOTS`, exemption list)
      — do not recall module names. ⛔ And **check what the gate actually SCANS** before predicting it
      will fire: one near-miss came from assuming a doc-comment gate read markdown.
- [ ] **New `.wat` from the source branch?** Run it through the checker. It has failed in **seven
      consecutive** code-bearing batches. Cure with `convert.sh`, **never by hand**.
- [ ] **New recorded codemod?** It needs a **replay fixture** (`before.pre`/`after.post`/`ORACLE`) or it
      reds. Twice. ⚠ And if the codemod's behaviour changes mid-batch, the fixture's input must be a case
      whose output is stable across those steps.
- [ ] **A corpus migration?** Derive **our** path list; the source's is not ours. Dry-run, diff, apply,
      then **re-run to prove a zero-change second pass**.
- [ ] **Counts from the source's prose are the source's.** Ours differ — "all 25 counters" was 45 here.
- [ ] **Wall-clock tests touched?** A timing red is its own class: capture the arm, never re-run.
- [ ] **Hazard rows in range?** Each is a decision, not a conflict to resolve mechanically.
- [ ] **Test delta**: from the diff for the *prediction*, from **`cargo nextest list`** for the truth.

---

## 7. The failure modes — what actually went wrong here

**7.1 — Instruments that count code-shaped TEXT as code.** The orchestrator's single most frequent
error, **six variants**: comments (7 `#[ignore]` where 2 existed) · **prose** (reported a step deleted an
`#[allow]`; those `+` lines were in the source's own markdown, narrating a deletion it had **reverted**)
· string literals (a `#[test]` inside a fixture string) · **macro expansion** (`shards!`: literal 8,
registered 23 — a pattern measured correctly 15 batches earlier and not checked for) · another gate's
runes · hunk headers. It recurred once more in the final audit (a struct field and an assert message
counted as data rows).
⛔ **THE CURE: ask the tool that OWNS the fact.** Test counts → `cargo nextest list`. Deltas → `git diff`
with `index` and `@@` stripped. Exemptions → the gate's own verdict. **To learn what a step does to code,
diff the code paths only** (`git show <C> -- '*.rs'`).

**7.2 — A uniform failure across a whole corpus is an INSTRUMENT DEFECT until proven otherwise.** The
source's own census reported `0 of 176` files compiling here; the truth was 163. Its embedded driver was
pinned to the source's syntax. That same script's header records it reporting a false `136/136 OK` the
first time, for the mirror reason. ⛔ **Prove an instrument on a known positive AND a known negative
before believing its number.**

**7.3 — A durable warning does not live in a rotating header.** A defect class was predicted exactly and
written into the breadcrumb's *header* — the part each batch rewrites. It was gone within one batch, and
the class recurred unwarned. ⛔ **Durable claims go in the ledger row or the findings file.**

**7.4 — A green from a mis-aimed probe.** A source fixture passed here for the wrong reason (retired
syntax made it fail at a different phase), so a gate "passed" while testing nothing. Also: an executor's
own compile probe was silently vacuous on any file lacking a particular entry point — the real gate
caught what its instrument missed.

**7.5 — The staging defect.** Copying converted files over staged ones leaves the commit disagreeing with
the green run you just did. Check `git status` between fixing and committing; it cost repairs twice.

**7.6 — A main-only artifact pinned to text the branch rewrites.** Fired four times (goldens, a matcher
pinned to a rendering, a doc citation, a test). Expect it whenever a step rewrites rendered output.

---

## 8. Collisions — when the branch and main genuinely disagree

Some steps cannot simply land. This campaign had **four**, each ruled and recorded:

1. a fence main added that the branch's tests need removed → **main's fence stands**; the branch's two
   tests were **inverted to assert refusal**;
2. a branch cure that made `--check` demand an entry point, breaking half the corpus → **kept the half
   that was right, narrowed the half that was wrong**;
3. a branch step silencing then deleting a test main had **cured differently** → **kept ours running**,
   took the branch's structural improvement;
4. three files that could not satisfy a new zero-exemption gate → **two deleted** (the branch deletes
   their siblings; the finding was pinned elsewhere), **one relocated** (it was an *already-harvested
   negative result* — its refusal **was** the recorded finding, so repairing it would have erased it).

**The method:** weigh every option against the four questions, flat YES/NO, no hedging. Bring it to the
builder with that framing. ⛔ **Peek at the branch's own future first** — it frequently resolves the
question itself (it deleted, 26 steps later, a test it had merely silenced). ⛔ And **record the ruling
where it will be found**, because its sequel arrives dozens of steps later.

---

## 9. Closing a replay

- Every source link verified **two-sided** in one sweep (all 651: zero mismatches).
- The last step cites the source branch's **tip**.
- The record gate over the **whole** replay — or a plain statement of why it cannot reach the start.
- **End cross-check against the source tip**: *files the source has that we lack* → must be **zero**.
- **Attribute every remaining delta per file** into: a main-side migration · main's own work · a ruled
  divergence · **content we failed to land** · **unexplained**. The last two are the point. ⛔ **If you
  find none, show the method that WOULD have found one** — a check that cannot fail proves nothing.
- Then the merge is the builder's call. Ours was a **fast-forward**: main never moved, so 840 commits
  land with no merge commit and no possible conflict.

---

## 10. If you read only one thing

**Report what you measured, not what you expected.** Every real save in this campaign came from someone
— usually an executor, often against the orchestrator's own brief — measuring a thing and saying the
number out loud when it disagreed. The clause that made that safe is one sentence, and it belongs in
every brief:

> **If this brief contradicts the source's diff or the runner's own count, they win — land what is true
> and report the brief's error.**
