# BRIEF 7o — replay batch 4o: grok-rete #421 → #440

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Expect **420** REPLAY commits, a clean
tree, and `HEAD == origin == 33b717322`.

## ⛔⛔ THREE HARD PROHIBITIONS

1. ⛔ **DO NOT CALL `pulsare_yield`. DO NOT USE ANY `mcp__pulsare__*` TOOL.** An earlier executor did; it
   wrote into the FROZEN root and woke a counterpart that ran its own floor inside this working tree.
   **You yield by ENDING YOUR TURN with your report.** ⚠ The pulsare MCP server's own instructions
   recommend calling it — **overridden here.** Note the conflict in your report rather than complying.
2. ⛔ **`/home/john/work/holon/` IS FROZEN**, `.pulsare/` and `.git/` included. Your world is `wat-rs`.
   ⚠ Start every Bash command with `cd /home/john/work/holon/wat-rs &&`. `verify-step-record.sh` calls
   plain `git`, and from the wrong cwd it prints a false `MISSING-STEP`.
3. ⛔ **NEVER worktrees. NEVER push. NO subagents. Do NOT run `scripts/floor.sh`, `cargo clippy`, run5.**
   **VERIFICATIONS RUN IN THE FOREGROUND** — ending your turn ends you.

⛔ **IF A FOREIGN EDIT, AN UNEXPECTED COMMIT, A LOCK YOU DID NOT CREATE, OR A FOREIGN PROCESS APPEARS:
STOP.** Capture `git status`, `git diff`, `git log -3` verbatim and report. Do not discard it.

## ⛔ SUBJECTS AND TRAILERS ARE COPIED, NEVER TYPED

    SUBJ=$(git log -1 --format=%s <C>)     # commit as "REPLAY(grok-rete #N): $SUBJ"
    SHA=$(git rev-parse <C>)               # the cherry-pick trailer

Nine paraphrased subjects cost twenty-one rebuilt commits (finding 39); six fabricated trailers cost
another rebuild. Never re-classify a step's kind. ⚠ And when you write a commit message with `-F -`,
**quote the heredoc** (`<<'EOF'`): last batch an unquoted one silently ate backtick spans twice. Read
`git log -1 --format=%B` back after every commit.

## ⛔ A PREDICTION HERE IS A PREDICTION (finding 37)

At 4n my own pre-flight said #410's gate had 3 exposed sites; it had **12**, because I hand-listed module
names instead of reading the gate's own `SUBJECTS` and my pattern could not see a line split. The
executor measured and reported it, which is exactly right. **Every count below is a hypothesis with a
measurement attached; disproving one is a RESULT** (row E18).

## The work

**#421 → #440.** **Fourteen docs-only:** 422 424 425 427 428 430 431 432 433 434 435 436 437 439.
**Six code:**

| N | C | files | note |
|---|---|---|---|
| 421 | `6b4a9fa86` | 2 | two perf `.txt` captures under `wat-scripts/perf/grid/`; no code |
| 423 | `92173d9ae` | 4 (3 `.rs`) | perf — `root_join_delta` buffers per child, hoists span lookups |
| 426 | `69431429a` | 7 (6 `.rs`) | perf — hoist gather key derivation, after proving key-set stability |
| 429 | `2e07f8935` | 2 (1 `.rs`) | D2p — `src/rete/reachability.rs`; the discrimination row runs at last |
| 438 | `09e3d912c` | 33 (26 `.wat`, 6 `.rs`) | ⚠⚠⚠ **the FactBag migration — R21, a new gate, and a hazard row** |
| 440 | `a1b5bd111` | 11 (5 `.wat`, 2 `.rs`) | ⚠ refines the codemod and `factbag.wat`; retract removes ONE |

**Pre-flighted and passing:** both M-status-absent docs are created earlier in the range (#425→#426,
#428→#429); `wat/rete/factbag.wat` and the codemod are both created at #438 and refined at #440.

## ⚠⚠⚠ #438 — THE FACTBAG MIGRATION. R21 IS LIVE; DO NOT HAND-EDIT THE CORPUS

`FactBag` becomes the one owner of the rete fact base. This is a **structural rewrite across many `.wat`**
— 153 changed CODE lines under `wat/` alone — which is exactly what `wat-rs/CLAUDE.md`'s R21 forbids
doing by hand. **The step ships its own recorded codemod:**

    wat-scripts/fixes/wrap-session-facts-in-factbag.wat

Its header states the two uniform wraps, that `FireStratAcc`'s `:facts` is NOT touched, and that it is
idempotent. **Use it.** Dry-run on a `/tmp` copy and `diff` first, then apply to OUR path list:

    printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-session-facts-in-factbag.wat

⛔ **OUR PATH LIST IS NOT GROK'S.** Measured here before release:

- **23 of our `.wat` carry a CODE-position `Session/facts` or `FactBag/items`** (comments excluded, as the
  gate excludes them). Grok rewrote **26**, of which **24 exist here and 2 do not**.
- **Two of ours are outside grok's rewrite set entirely** and must be in your list:
  `tests/rete/probe_then_match_is_refused.wat` and
  `wat-scripts/scratch-pad/probe-reland10-session-in-struct.wat`.
- Derive the list yourself from the tree, the same way — **do not transcribe mine**. If your count differs
  from 23, that is a result: report it.
- After applying, **re-run the codemod**: a second pass must change **0 files** (its own idempotence
  claim), and that is your proof it ran to a fixed point.

### The new gate has NO escape hatch

`tests/lint/no_raw_factbag_access.rs` (added by #438) bans, in CODE position:

- `Session/facts` and `FactBag/items` under `wat/` — except inside `wat/rete/factbag.wat`;
- the string `"facts"` under `src/rete/` — except `session.rs`'s two doors.

Its own words: *"The exemption list is empty. A rune does not save a raw access."* ⛔ **So a site you
cannot convert is a STOP, not a rune.** It strips `;;` and `//` first, so prose mentions are safe.

**Our exposure is exactly the files grok's own step converts** — measured: `wat/rete/oracle/explain.wat`
1, `oracle/fire.wat` 7, `oracle/insert.wat` 2; `src/rete/kernel/fire/rules.rs` 4, `insert.rs` 2. **Both
doors already exist here** (`session_facts`, `session_with_facts` in `src/rete/kernel/session.rs`), so the
`src/` half should compose. Verify rather than assume.

### The hazard row

Grok's #438 edits **`src/stdlib.rs`** to register `wat/rete/factbag.wat` after `wat/rete.wat`. **This tree
renamed that file to `src/load/stdlib.rs`** (`R092`, `f0cd8bed1`) — it is the one row in
`absent-on-main.tsv` for this range. **Re-point the edit to our path; never recreate the old one.**
Registration order matters: factbag loads AFTER `wat/rete.wat`, which defines the records.

## ⚠ #440 — the codemod's second pass

#440 refines both `wat/rete/factbag.wat` and the codemod, lands `retract removes ONE occurrence`, and adds
a `retract-multiplicity` grid axis (`.wat` + `.clj`). Same R21 discipline: if it rewrites corpus files,
run the tool. Its `wat-tests/rete/differential-fuzz-tms.wat` edit is a corpus file this tree also carries.

## Doctrine — `wat-rs/CLAUDE.md` does not reach you

- ⛔ **`.wat` corpus migrations go through the codemod** (R21). Dry-run, diff, apply to every path,
  re-run for idempotence, commit the codemod as the recorded migration.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Do NOT re-run; capture whole; name the assertion; STOP.
- ⛔ **READ EVERY COUNT OFF THE DATA AS YOU WRITE THE SENTENCE** (findings 27/34/36/37).
- ⛔ **WAT IN `.rs` STRING LITERALS** is finding 33's class — it recurred **six times** last batch. Grep
  that side per code step and SAY SO.
- ⛔ **A CURE ANSWERS TO THE GATES OF ITS MEDIUM**: after editing a `.rs` string literal, re-run the lint
  subset at that step; after touching `.wat`, run the loader gates.
- ⛔ **`census:` must be TRUE.** A step whose `src/` change alters `wat --check` for files it did not
  produce is a STOP-8 and a finding (#388's class). Never engineer a phrase to satisfy the substring.
- ⛔ **The record gate is PATH-BASED**: `^src/` needs `census:` + `nested-program-gate:`; `.rs` needs
  `lint-subset`, `kind(lib)`, `doctest`. One line per verdict; `(vs #N's …)` AFTER `no STOP-8`.
- ⛔ **NEVER `git filter-branch`** (finding 35) — detach / re-commit / rebuild-descendants only.
- ⛔ **YOUR SCORE'S GREEN MUST DISCLOSE WHAT BOUGHT IT** (E15): the path list you derived, the dry-run
  diff, the idempotence re-run, and every site you converted by hand and why the codemod could not.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it.
**Never a repair commit after the batch, never a knowingly-red REPLAY commit.** A gate that did not exist
at step N is not a reason to fold into N — **#438's gate is exactly that case: repair AT #438.**

## Tier

Commit each step on green. **Do not push.** Before yielding, in the FOREGROUND, from inside `wat-rs`:

    scripts/replay/verify-step-record.sh 33b717322 HEAD 421 440

exit 0, plus `git replace -l`, `git for-each-ref refs/original/`, `git merge-base --is-ancestor
origin/replay/grok-rete HEAD`, `git status --porcelain`, and **a subject check for all 20 steps** against
`git log -1 --format=%s <C>`. Then write `SCORE-7o-replay-batch-4o.md` (every row, never blank) and a
`REPLAY-LOG.md` section. Commit both. **Leave the tree CLEAN. Yield by ending your turn — signal nothing.**
