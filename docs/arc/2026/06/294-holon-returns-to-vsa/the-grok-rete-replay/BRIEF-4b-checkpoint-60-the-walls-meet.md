# BRIEF 4b — checkpoint #60: two walls meet the other side's content; fold the repairs, close 2a4's two flaws, and census every step

> Batch 1 replayed #11–#60 (50 steps) and stopped at STOP-4: the #60 checkpoint floor is red,
> 5 tests (`.floor/2026-09-14T01-43-05Z`, `SCORE-4-replay-batch-1.md`). Diagnosed by the
> orchestrator; both repairs PROBED together (below). Finding 18.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Never use worktrees.

## Why — measured

The two defects are one class: **a wall from one side meets content from the other side for the
first time at the merge.** Neither side's commit is wrong on its own side, and the per-step gate
never looked past the files the step produced.

1. **Arms 2–5 — grok-rete's rete wall meets main's fmt rules.** #50 (`4354afc6e` → `3ee40f787`)
   adds `UnconsumedWrapperBind`: a variable declared fresh inside a `:not` must be consumed within
   that `:not` (#50's message: *"Declared must be consumed"*, the builder's own arc 109 precedent;
   the author fixed their 6 violations *"by dropping the dead bind"*).
   `wat-scripts/fmt/rules/{kwargs,defrecord}.wat` exist only on main — created after the fork
   `de827fb4c`, never on grok-rete — and carry 24 such dead binds across 10 rules (23 in
   `kwargs.wat`, 1 in `defrecord.wat`; `bootstrap/era/probe-T/sites-60.txt`). Every program that
   loads them dies at startup; the other 17 files the census flags only load them. Example,
   `kwargs.wat:31`: `(:wat::grep::Node (?more <- :id) (?p <- :parent) (?mi <- :index))` inside a
   `:not` — `?more` is used nowhere.
2. **Arm 1 — main's separator lint meets grok-rete's config code.** `tests/lint/one_variant_separator.rs`
   landed on main `7ccce48ba` (2026-09-10); grok-rete never had it. **#60** (`d039b29e5` →
   `2528967fb`, not #59 as SCORE-4 says) adds `src/config.rs:415`
   `wat_reader::identifier::leaf(head).starts_with("set-")`: the last segment of a config setter
   head, a namespace test, not a variant. The lint's closed category `namespace` is its designed
   door for exactly this; `src/declare/register.rs:1702` and `src/edn/render.rs:1656` use it.

**The probe** (`bootstrap/era/probe-T/probe-60-fix.sh`, on the tree, reverted): drop the 24 binds
by the wall's own spans (each guarded: the text at the span must be `(?<var> <- :<field>)` with the
reported var) + the rune → all 19 census files `--check` rc 0, no `UnconsumedWrapperBind` left, and
the 5 tests PASS — including `doc_row_pprintln_matches_byte_golden`, so the fmt output did not move.

## A — the repairs go INTO the steps that need them

This branch becomes main, so no REPLAY commit may be knowingly red. #50–#60 are unpushed;
rebuild them:

1. `git tag replay-pre-fold replay/grok-rete` (local; the old tip, this brief's commit included),
   `git switch -c fold ea9aedd2e` (#49). Your uncommitted `SCORE-2a4.md` / `SCORE-4…` /
   `REPLAY-LOG.md` ride along unchanged.
2. `git cherry-pick 3ee40f787` (#50, no `-x`: its message already carries the trailer). Write the
   recorded migration `wat-scripts/fixes/drop-unconsumed-negation-bind.wat` (B below), run it over
   the two fmt rule files, `git commit --amend` — the body gains a `Composition:` paragraph naming
   main's fmt rules, the 24 sites, and the migration.
3. `git cherry-pick 3ee40f787..1fd0f5b21` (#51–#59), then `git cherry-pick 2528967fb` (#60); add
   the rune on the line above `config.rs`'s `leaf(head)` line —
   `// rune:lint(one-variant-separator, namespace) — last segment of a config setter head, not a variant`
   — and `git commit --amend` with a `Composition:` paragraph.
4. `git cherry-pick 2528967fb..replay-pre-fold` — the commits after #60 on the old tip (this
   brief's own commit).
5. `git branch -f replay/grok-rete fold && git switch replay/grok-rete && git branch -d fold`.

**Its proof:** `git diff replay-pre-fold HEAD --stat` names exactly the two fmt rule files
(24 lines), `src/config.rs` (+1), the new migration and its fixture — nothing else; the
`REPLAY(grok-rete #` subjects #11–#60 are unchanged, 50 of them.

## B — the recorded migration: the wall names the sites, the tool only removes them

`wat-scripts/fixes/drop-unconsumed-negation-bind.wat`:
- **Input (stdin):** the sites the wall reports, as `path:line:col:var` strings — taken from the
  `--check` report's structured `:errors` entries (`#wat.rete/UnconsumedWrapperBind` →
  `:var`, `:span` `:file :line :col`), never re-derived. A second implementation of
  "fresh and unconsumed" in wat is two slots that can disagree.
- **Edit:** at each site, remove the child list whose `ast-span` starts at `line:col` and one
  adjacent space — via `wat/fix.wat`'s span edits (`fix-source` / `fix-text-apply`).
- **Guard:** the node at the span must be exactly `(?<var> <- :<field>)` with the reported var;
  anything else refuses loudly and edits nothing.
- **Fixture:** `wat-scripts/fixes/replay/drop-unconsumed-negation-bind/{before.pre,after.post,stdin,ORACLE}`
  per `tests/cli/every_recorded_migration_replays.rs` (the ORACLE cites the header spec and #50's
  message). Idempotent: a second run changes nothing.
- **Postcondition:** the wall reports zero `UnconsumedWrapperBind` tree-wide.

## C — 2a4's two flaws (verified in c184348f7; no replayed output was affected)

A new commit on top, `2a4b`:
1. **A replaced enum keeps its OLD variant singletons.** `TypeEnv::retract_for_door_replace`
   (`src/types.rs:708`) removes only the enum's own name; `register_variant_types` skips a variant
   whose singleton already exists (`src/types.rs:1131-1143`, `if self.get(&fqdn).is_some() { continue; }`).
   So in the door's copy, `E.V` keeps the old fields and a removed variant survives. The door's
   output is unaffected today (its `TypeInfo`s come from the parent), but the copy is not the world
   the file declares. Retract the replaced enum's variant singletons too. Fixture: `V` changes
   fields and `W` is removed → the copy's `E.V` singleton answers the NEW fields, `E.W` is gone.
2. **`stdlib-source-path?` misclassifies 5 tracked files as stdlib.** Its `contains "/wat/"` clause
   (there only because `convert.sh` hands absolute out-dir paths) admits
   `crates/wat-edn/wat-edn-clj/wat/shared.wat` and `examples/{console-demo,with-loader}/wat/*.wat`.
   Measured: the tracked `wat/**/*.wat` are EXACTLY the 63 `STDLIB_FILES` paths, so a repo-relative
   `wat/` prefix is exact. The running binary's list cannot decide it (at #22 `wat/gen.wat` is new).
   So: the door's callers receive repo-relative paths (`convert.sh` runs the codemods from the
   out-dir); the predicate is `starts-with "wat/"` and REFUSES an absolute path loudly; a gate asserts
   tracked `wat/**/*.wat` ≡ `:wat::stdlib::sources`' paths, and goes RED once under a mutation.

## D — every step censuses the tree (the class, not the case)

From #61 on, a step's gate adds two rows to BRIEF-1 § "One step" 3:
- **every step with a `.rs` change:** `cargo nextest run --release -E 'binary(lint) - test(every_wat_scripts_file_loads_on_the_current_runtime)'`
  (138 tests, seconds; would have caught arm 1 at #60);
- **every step whose build changed the `wat` binary (sha256 before/after) or that changed any `.wat`:**
  a whole-tree `--check` census diffed against the previous step's (would have caught arms 2–5 at #50).
  `scripts/replay/census.sh`: `git ls-files '*.wat'` → `./target/release/wat --check` at `-P32`,
  one `path rc` line each, into an ignored directory. Measured on `2528967fb`: 2047 files, 30.6 s,
  219 failing at the baseline. **A file going rc 0 → non-zero that the step did not produce is
  STOP-8.** 200 of the remaining 591 steps qualify: ~1 h 45 m over the whole replay.

Found at its step, a composition repair is part of that step's commit, and no rebuild is ever
needed again.

## The bar

- A's proof (above); the checkpoint floor at the new #60 GREEN, clippy 0.
- B's fixture gate green; the migration's postcondition; `pprintln_doc_row` byte golden unchanged.
- C: both fixtures RED under the mutation that removes each fix (captured), GREEN with it;
  `run5.sh` unchanged (0 losing; chain vs main ≥ 1370).
- D: `census.sh` run at the new #60 is the batch-2 baseline. Its own control, free: diffed with
  the new #60 census as the previous and `bootstrap/era/probe-T/census-rc-2528967fb.txt` (the old
  tip, measured by the orchestrator) as the current, it prints STOP-8 for exactly the 19 files the
  wall broke.
- Floor + clippy after C.

## STOP triggers — rejections; report verbatim

- **STOP-1:** any cherry-pick in A conflicts, or A's proof names a file outside its list.
- **STOP-2:** the migration's guard refuses a site, or the postcondition is not zero.
- **STOP-3:** a floor is red — paste the whole block, do not re-run.

## Tier

Commit on green. **Do not push.** Yield with `SCORE-4b.md` and an updated `REPLAY-LOG.md` (new
hashes for #50–#60). Batch 2 is a separate brief.
