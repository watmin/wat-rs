# SEAM — the ONE live breadcrumb. 2026-09-11. ⛔ **YOU ARE ON A MERGE BRANCH, NOT MAIN.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git rev-parse --abbrev-ref HEAD      # expect replay/grok-rete  — NOT main (merge/grok-rete is the REFERENCE)
git status --porcelain               # expect EMPTY; a live peer strike makes it dirty
ps -eo pid,etime,cmd | grep wat | grep -v grep     # ⛔ a codemod still running? NEVER start a 2nd
ls -la bootstrap/ && cat bootstrap/wat-main-*.PROVENANCE     # the escape hatch — must exist
cat /home/john/work/holon/.pulsare/to-claude       # has grok scored?
```

## ⛔⛔ THREE FACTS THAT WILL BITE A FRESH SELF WITHIN MINUTES

**1. On `merge/grok-rete` (the REFERENCE, not the replay branch), `./target/release/wat` CANNOT
BOOT.** On `replay/grok-rete` it boots: the pilot built it and ran floors on it (5404/5404). The merged stdlib (grok-rete's
`wat/gen.wat`, `bracket.wat`, `query.wat`, …) carries pre-rename spellings that the merged
runtime's purity gate refuses at startup. **Every codemod dies instantly.** Use the escape hatch:
`bootstrap/wat-main-a3218644d` — main's binary, stdlib frozen in at build time, PROVEN to convert
a pre-flip file. The tracked record of it lives in `.gitignore` (the `bootstrap/` comment), because
the ignored dir already lost one binary and a `/tmp` copy died in a WSL crash.

**2. THE CODEMODS ARE NEVER THEIR OWN INPUT.** Running the chain over `wat-scripts/fixes/` let
`fmt-head-fqdn-to-clojure` rewrite a string literal INSIDE `positional-ctor-to-map`
(`":wat::core::unquote"` → `"wat.core/unquote"`), disarming its predicate; four steps later it
emitted `{:pred ~}src` and corrupted `wat/query.wat`. Exclude `wat-scripts/fixes/` from every path list.
`[[feedback_a_tool_is_never_its_own_input]]`

**3. TIME A TOOL ON ONE FILE BEFORE RUNNING IT ON THOUSANDS.** `positional-ctor-to-map` costs
**~15 s/file** (56 s/file on dense files) — a ~9-hour run, killed at 4h08m. Then I launched a SECOND
run without checking the first had exited: two processes doing read-modify-write on one corpus.
`[[feedback_time_a_tool_on_one_item_first]]`

## WHERE THE WORK IS

```
main              a3218644d   FROZEN · PUSHED · floor 5373/5373 GREEN · clippy 0     do not touch
origin/grok-rete  37528f6e0   FROZEN
merge/grok-rete   (this)      merge committed, ALL TARGETS COMPILE, corpus NOT migrated
```

- `4366f4fbd` — grok's merge: 99 conflicts (35 .rs · 39 .wat · 23 .edn · 1 toml · 1 .bad).
  merge-base `de827fb4c` (2026-08-24); main +911, grok-rete +651.
- `db60b092a` — five home-retargets (`load::loader::`, `normalize_rust_source_span_lines`,
  `value_to_edn_with`); `cargo build --release` passed the merge but NOT `--all-targets`.
- ⛔ **Three warnings deliberately UNSILENCED** (`eval_lower`, `field_types`, one test import) —
  refactor residue from main moving rete dispatch to `#[wat_intrinsic]`, but whether those paths
  FULLY supersede grok-rete's is a rete judgment. Prefixing an unused var whose doc calls it
  load-bearing would hide a dropped check. **They go back to grok.** Clippy is red until then.

## ⬜ THE MIGRATION — TWO PHASES, AND WHY

28 recorded codemods landed on main after the split; grok-rete's corpus saw none of them. Replay
in **landing order** (= dependency order). Regenerate the order — the `/tmp` copy is gone:

```bash
for f in $(git diff --name-status de827fb4c main -- wat-scripts/fixes/ | grep '^A' | awk '{print $2}'); do
  echo "$(git log --format=%ci --diff-filter=A -1 de827fb4c..main -- $f) $f"; done | sort | awk '{print $4}'
```

**PHASE 1** — `bootstrap/wat-main-a3218644d` over the corpus INCLUDING merged `wat/`, EXCLUDING
`wat-scripts/fixes/`. That should make the merged stdlib bootable → rebuild.
**PHASE 2** — the rebuilt merged binary, over the same list. Main's binary does NOT know
grok-rete-only reserved types (`:wat::rete::FireOutcome` / `CompileOutcome` / `InsertOutcome` …) —
they print `UNRESOLVED` in phase 1. **Phase 1's UNRESOLVED lines are phase 2's worklist.** Do not
estimate that number with grep.

## ✓ LANDED — A′ (`bbcb70e38`): `:wat::`/`:rust::` resolve ONCE, every local name PER FILE

Honest by construction (`registration.rs:165`: equivalent-before-reserved). **Not**
`is_reserved_prefix` — `:$bound::` is per-scope and deliberately excluded, documented in the
codemod. Every acceptance row re-verified by the orchestrator: gate A == B 20/20 (mtime-controlled),
the collision pair no-halt and OLD-identical, ReservedPrefix refused on `:wat::` AND accepted on
`:u::`. 3.1× on the same 20 files. Option B's halting version is gone.

## ✓ LANDED — (c) (`4abd3d19e`): a local path is resolved only if THIS file could declare it

Exact, not heuristic — **audited independently: 342/342 skipped paths resolve to None** (gate 265,
samples 77; the audit dies on the first Some, so rc=0 is the signal). One predicate
(`keep-local-ep?`, `:481`) drives both skip and audit. Log: `bootstrap/audit-c-gate-20260912T020655Z.log`.
⚠ Grok's `/tmp/pctm-c` kept only INPUT path vectors — no output from its audit survived.
Timing, same files: n=5 2.5×, n=20 2.1×; at small n the once-per-run invariant pass is the floor.

## ⛔ PHASE 1 IS BLOCKED — do NOT run the chain. Three premises under it measured FALSE.

The first run (`bootstrap/chain-runs/2026-09-12T02-15-59Z/`) changed **0 files** and stopped at
step 3. Each finding below can be re-derived with the command beside it.

**1. 11 of the 28 codemods are DEAD on main's binary.** These are steps 1 and 3–12, the grep-rules
ones: 40 rules, and every one joins `Named` and `Span`. (Step 21, `node-kind-string-to-enum`, is an
AST walker with 0 rules. My first partition counted it because `grep ':wat::grep::Node'` matched a
STRING LITERAL inside it; counting `defrule` corrected it.) Since `0b5742cc7`, main's extractor folds every `Named` name to clojure form
(`git show a3218644d:wat/grep.wat | sed -n 239p`).
- A `starts-with ":wat::…::"` rule matches nothing and exits rc=0.
- The five that `0b5742cc7` migrated do match, then splice the canonical `?n` as old-text, and
  `fix-text-apply` refuses (step 3's log).

The control pair, on a 2-line probe:
- `bootstrap/wat-pre-named-0e3a351f3` running `git show 0e3a351f3:wat-scripts/fixes/rename-core-string-to-string.wat`
  **renames** it.
- Main's binary running today's copy of that file renames **nothing**.

No gate replays a recorded migration. `grep_programs_still_match.rs` covers `wat-scripts/grep/` only.
`[[feedback_a_recorded_migration_is_pinned_to_its_fact_model]]`

**2. Landing order is not scope.** Steps 2, 20, 21 and 22 migrate TOOLING (codemods, fmt rules,
grep programs), not the corpus. Read each header. `fmt-head-fqdn-to-clojure` rewrites ANY
`":wat::…"` string, so over grok's stdlib it would rewrite data strings.

**3. My merge brief's `.wat` rule was false.** `the-grok-rete-merge/BRIEF-merge-grok-rete.md` says:
*"take grok-rete's CONTENT; main's side is almost entirely codemod output."*
- **34** conflicted files are byte-identical to grok's blob at `4366f4fbd`, and main's HAND content
  in them is gone. `wat/grep.wat` lost `NodeKind`, `Written` and the canonical fold.
- **16** resolved to main's blob. Grok's side was dropped; the SCORE says why for each `.rs`.
- **10** modify/delete conflicts went to deletion.

Recompute: `git merge-tree --write-tree --name-only 1f39db790 37528f6e0` gives the 99. Then compare
each file's blob at `4366f4fbd` with both parents. `[[feedback_a_landing_commit_is_not_codemod_output]]`

**4. The `.rs` side is RED on main's own walls, whatever happens to the corpus.**
`cargo nextest run --release --test lint` gives 312/350 (log: `bootstrap/lint-merge-973a391e9.log`).
- **Corpus not migrated (27; these clear only after a WORKING chain).** Every one dies at startup on
  `:wat::core::string::interpolate` in the merged stdlib:
  - `wat_scripts_fixes_load` 761/761
  - `rete_compile_gate` ×16 (338/338 reasons are startup)
  - `every_rete_name…resolves` (98, all pre-rename `:wat::rete::core::*`)
  - `every_ungated_wat_checks` · `docs_wat_loads…` · `probe_arc277_*` ×6 · `diagnostic_output…nested…`
- **The merge broke source (11):**
  - `one_variant_separator`: 21 sites. Grok's `expr_ir/` split resurrected the `format!("{}::{}")`
    that main's door `9ee28f3b4` had removed.
  - `ast_kind_nodekind_sync`: *"defenum :wat::grep::NodeKind not found"*.
  - `no_bare_is_err` 6 · `no_error_flattening_helper` 3 · `no_stale_path_in_doc` 4
  - `rete_citation_resolves` ×2 · `rete_header_claims…enum_variant_ctor` (3 callers ≠ 4 documented)
  - `prose_in_rust_does_not_attest_a_name` · `no_new_broken_doc_link` 9 · `every_walking_gate…non_vacuity`

## ✓ RULED 2026-09-12 — replay grok-rete ONE COMMIT AT A TIME onto a branch off main

The builder:
- *"main is upgrading syntax, the others are upgrading subsystems"*
- *"we step forward one commit at a time"*
- *"correct but slow is better than fast and wrong… willing to accept the wall clock and token cost
  if it means we don't lose any of our hard work."*

`[[project_merge_doctrine_syntax_vs_subsystems]]`

- **Ownership:** main owns ALL syntax, plus all behaviour outside the branch's subsystem. The branch
  owns its subsystem's behaviour. A shared component takes main's version, with grok's change
  re-expressed on top of it.
- **Done = two gates:**
  1. Every grok-rete test exists in main's syntax and passes at the commit that introduces it, or
     carries grok's written reason.
  2. Main's walls are green: lint, then floor, then clippy.
- **The shape, measured:** 651 commits, linear, 0 merges. 376 are docs-only and 275 touch code. 113
  touch a file main changed (91 of those `.rs`). Commit #1 already touches `.wat`.
- **One step:**
  - docs: cherry-pick
  - `.wat`: migrate the before and after versions with main's codemods, then `git merge-file`
  - `.rs`: cherry-pick, then re-express on main's shared components (rerere; the merge SCORE is a crib)
  - wat embedded in `.rs`: by hand, per step
  - gate: a build plus the commit's own tests; the floor at stone checkpoints
- **`merge/grok-rete` is now the REFERENCE, not the destination.** Its resolutions are a crib, and
  its final tree is the end cross-check. Its 8 tree-wide-missing grok tests
  (`bootstrap/merge-audit/`) are the first loss signal the replay must NOT reproduce.

## ⬜ NEXT

1. **Step 0: every recorded migration works on today's binary.** The builder ruled **all 94** replay
   against a fixture (2026-09-12).
   - The package is `the-grok-rete-replay/`: DESIGN, `BRIEF-0a-rewriters-read-what-is-written.md`,
     `EXPECTATIONS-0a.md`.
   - **0a ✓ LANDED:** `Written :text`, then the 11 codemods on `Written`, the 45 literals restored,
     and the replay gate `tests/cli/every_recorded_migration_replays.rs` with 13 fixtures + 83
     ledgered. It was grok's strike; the orchestrator re-ran every EXPECTATIONS row itself, and
     drove the gate red four ways on a different codemod than grok's. The orchestrator also fixed
     two things: the gate's count pins are gone (it freezes names), and the keyword fixture gained
     near-misses.
   - **✓✓ STONE 0b CLOSED — `c92503d98` (grok's 2d) + `47efbd2af` (orchestrator fixes), PUSHED.**
     - **All 95 recorded migrations replay:** 90 fixtures + 5 unreadable-preimage runes. Each has
       an `ORACLE` the gate verifies against git: history on main's history, spec quoted from the
       header, and every changed before- and after-line accounted for.
     - `git_show` and `git_cat_file_exists` fail loudly.
     - `rename-record-def-to-defrecord` was restored (ruled).
     - Floor 5391/5391 with no SLOW (`positional_ctor` has its own envelope, 63.2 s); clippy 0.
     - The orchestrator reproduced (l)(m)(n) on its own runs, (n) on a different fixture than grok's.
   - **STARTED 2026-09-13 (builder: "pulsare"):**
     - **pilot:** `BRIEF-1-pilot-first-ten-commits.md`, grok-rete #1–#10. It builds a tracked
       `scripts/replay/` (chain order derived from git + overrides; a SCOPE-honest converter).
     - **the composition check:** `bootstrap/composition/run.sh`, with the FROZEN binary
       `bootstrap/wat-replay-base-8a5b7eb20` and the exported tree `bootstrap/composition/tree-8a5b7eb20`,
       at nice 19, never touching the tree. Timed on one file first: 54 s, 46 s of it positional-ctor.
       The baseline run (landing order, 1489 files) is running; runs land in
       `bootstrap/composition/runs/`.
   - **✓ PILOT VERIFIED — grok-rete #1–#10 replayed** (`964e82c3e`…`5c51f9a8a`, SCORE `7be13903e`).
     Re-run on the orchestrator's own runs:
     - each trailer resolves to its grok-rete #N, and no file falls outside its source commit;
     - #3 re-expressed on main's door (its one `rsplit_once("::")` is a doc comment, `matcher.rs:114`);
     - `wat/cache.wat`'s 3-way identity holds (byte-identical);
     - the standalone DuplicateDefine is pre-existing (the base binary gives it too);
     - floor **5404/5404**, every named step test PASS; clippy 0.
     - ⚠ watch: `wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family` went SLOW at
       64.2 s (>60 s warn) under contention from two niced jobs. It passed, and grok's floors did not
       flag it; #6 and #9 touched that file.
     - The pilot's extrapolation for the remaining 641 is ~16.5 h.
   - **The composition findings are TRACKED** in `the-grok-rete-replay/FINDINGS-composition.md`:
     - 1: a sweep co-updated an earlier tool.
     - 2: an eval-based tool runs on today's substrate.
     - 3: one unmigrated body poisons type resolution. (B), type-decls-only, at scale: 403 → 16
       UNRESOLVED, 0 regressions. The 16 split into 1 that matches main, 9 of main's hand content, and
       6 from two (B) gaps (acronyms; a nested program's scope), each closed by a probe.
     - 4: positional-ctor's UNRESOLVED count is mostly noise, from the case rule `pascal-leaf?`.
     - 5: the LANDING's match-arm leaked a type across files (reproduced), and main carries
       `{:deps addr}` in `probe-m1-ann-erase2`, which is also broken on both sides (undeclared `CMsg`).
     - **✓ RULED 2026-09-13 ("all recommended"), six decisions, each argued with the four questions
       in the main chat:**
       1. (B) is the LADDER: today's declarations first; pure type declarations + `declare-acronyms`
          (any depth) only when those cannot answer. Measured: 403 → 15, 0 lost (`probe-L/`).
       2. `variant-separator` decides what a variant is from the program's declarations + the
          registry, through the same ladder. The frozen 382-pair census is retired.
       3. The ladder lives ONCE, in `wat/fix.wat`, shared by the tools. No per-tool copies.
       4. A GATE checks every nested program literal (`(:wat::core::forms …)`). Its own stone,
          after 2b, whose first act is the census of the 164.
       5. `surface-field-dispatch.wat` (a broken docs probe that grok-rete modifies) is fixed when its
          grok-rete commit replays. Not deleted.
       6. **BRIEF-2b RELEASED ("pulsare 2b")** — `BRIEF-2b-main-teaches-what-it-refuses.md` +
          `EXPECTATIONS-2b.md`.
     - **✓ 2b VERIFIED — `f2e0ac26b` (grok's SCORE-2b).** Re-run on the orchestrator's own runs:
       - E1: 7/7 site tests PASS;
       - E2: mutated a DIFFERENT site than grok's (`check.rs:7755`), and exactly
         `list_constructor_pattern_teaches_dot` went red, naming the site; restored;
       - E3: no code string teaches `::` (own census);
       - E4: the one-variant-separator lint PASSES;
       - E5: the old c03 template reproduces RED; E6: `contract_03` GREEN;
       - E7: both `probe-m1-ann-erase*.wat` print `echo:z`;
       - E8: 20 files, within the blast radius plus fixtures;
       - clippy 0; floor **5411/5411**, no SLOW (`.floor/2026-09-13T04-58-38Z`, run uncontended: the two
         pc2 wat processes were SIGSTOPped for the floor, then resumed).
       - Three known flaws it surfaced are in FINDINGS § "After 2b": 3 refusal arms unreachable from wat
         (dead code pinned by text) · ~21 doc/comment lines still teach `::` (SCORE listed 4) · c03's
         `variadic-wrap` now hardcodes `:wat::core::i64`.
     - **✓ RULED 2026-09-13: BUILD THE DECLARATION DOOR** (builder: "we build it", after the four
       questions came back 4 YES). Findings 9–12: every `eval-with-defs!` is two full startups, and the
       exact wat-side workaround caps at 2.4× per commit.
       - **`BRIEF-2a1-the-declaration-door.md` + `EXPECTATIONS-2a1.md`** — released ("pulsare"); grok
         scored `175b49ea9`: `:wat::runtime::declared-types` → `:wat::runtime::DeclaredTypes`
         (`Ok [types]` / `Refused [form cause]`), 12–17 ms a call.
       - **ORCHESTRATOR RE-RUN of 2a1:**
         - floor 5421/5421 uncontended (`.floor/2026-09-13T07-32-14Z`); clippy 0;
         - 19 tests pass;
         - own mutation: `preregister_acronyms` neutralised → exactly the acronym test RED; restored;
         - the verb called from wat (`bootstrap/era/probe-R/door.wat`): the acronym file is correct.
       - ⛔ **REFUTED** (`BRIEF-2a1-REFUTE.md`, waiting for "pulsare"):
         - R1: c3's top-level macro call `(:t::mk :demo)` is DROPPED (hand-list filter
           `is_declaration_form`), so the verb returns NO types, silently;
         - R2: the hand list needs a check that fails on an unadmitted type-registering head;
         - R3: `Refused.form` is always the program's first form;
         - R4: the oracle covers 1 of the 4 cases (the acronym file loads).
         A pure verb: expand a program's declarations with the stdlib macros, register them into a
         fresh copy of the stdlib registry, and return the `TypeInfo`s. No startup, no body check.
         Probe first, inside the crate.
       - The exact wat-side batched ladder is the measured FALLBACK (`probe-Q/run4.sh`: 0 files differ,
         UNRESOLVED 15, 0 constructors lost). It is also the bar 2a2 must reproduce.
       - The 2a draft becomes **2a2**: the codemods ask the door once per program.
     - (superseded) NEXT: the chain-tooling brief (2a): the ladder door, the scope rule, the variant decision,
       positional-ctor reporting by registry (not case), step 14 reporting a splice, and `convert.sh`
       giving `one-param-spec` the corpus context. It waits for the positional-ctor ladder A/B and the
       chain under the ladder (`probe-L/pc2.sh`, running on copies, ETA ~06:25Z).
       - DRAFTS (untracked, `bootstrap/pending/`; each is derivable from FINDINGS if lost):
         `BRIEF-2a-draft.md`, and `BRIEF-3-draft-every-spawned-program-starts.md` (ruling 4's gate).
       - Ruling 4's census is DONE and classified (`3acea9e98`, FINDINGS § Finding 8). 141 literals.
         At least 9 broken child programs hid under a green floor: 7 arc-170 probes that predate the
         RecvOutcome/SendOutcome walls, plus `erase{,2}` (fixed in 2b). `--check` is stricter than the
         child's real path (`src/process/verbs.rs:429-433`).
     - **✓ RULED 2026-09-13: (B)** ("B has been decided"). The eval-based codemods learn field names
       from the type declarations of the program that holds the site, on today's binary. The era
       binary (A) is rejected: it reproduces the landing's defects and runs outside stone 0's gate.
       Next: a brief for grok. The v2 RESULT gives positional-ctor's residuals and the census for
       finding 5.
   - ⛔ **COMPOSITION FINDING 2 (baseline run, landing order): the chain STOPS at step 23.**
     - `match-arm-to-bracket-map-pattern` rc=2 on `tests/process/probe_arc278_init_crash_reason.wat`,
       with `assertion-failed! … positional form is retired`.
     - Its `try-type-of` EVALUATES the file's declarations on TODAY's runtime, which already refuses a
       form that a LATER step (24, `assertion-failed-to-kwargs`) migrates.
     - **The rule:** eval-based codemods (match-arm, positional-ctor) run AFTER the pure rewrites
       whose old forms today's runtime rejects.
     - `bootstrap/composition/order-v2.txt` = landing order + two moves, each with its evidence
       (`order-v2.README.txt`): assertion-failed → before match-arm; bare-variant → before
       positional-ctor. It is running.
     - The pilot's `chain-order.overrides` must carry BOTH moves. grok will hit this as STOP-2 if a
       pilot `.wat` evaluates a positional `assertion-failed!`.
   - **0b batch 2c (`d1c47b2a4`) GREEN** (floor 5391, clippy 0). The ORACLE arm is real: (i)(j)(k)
     reproduce RED, and all 67 history citations are on main's history. Still open for a batch 2d:
     - `git_show` swallows a bad commit/path (a bogus path passed rc=0, reproduced);
     - mixed fixtures skip their before-line check;
     - `namespace-bare-top-level-names` REPRODUCES its landing on 5 of 6 files, so it must be
       history, not spec;
     - `rule-record-to-defrule`'s stated reason is wrong: its landing inputs are angle-bracketed
       (lex error), not a no-op;
     - `rename-record-def-to-defrecord` was corrupted by `dedcb74a7` (a self-migration).
       **RULED 2026-09-12: RESTORE.**
     - **Briefed as batch 2d** (`BRIEF-0b-REFUTE-batch-2d.md`). 2d closes stone 0b.
   - ⛔ **REPLAY HAZARD, CONFIRMED on today's binary:** `positional-ctor-to-map` (landing-order line
     25) now matches `:wat::core::Option::Some`, because `49f03f179` qualified it. A bare
     `(:wat::core::Some 1)` prints UNRESOLVED and stays unchanged; `bare-variant-to-qualified`
     (line 26) then leaves `(:wat::core::Option::Some 1)` POSITIONAL. **Landing order is not the
     dependency order of TODAY's tools.**
     - **The replay's FIRST requirement:** a chain-composition check. Run the whole chain over each
       base-era `.wat` and compare it with main's version; every residual is a main hand-edit or a
       chain failure, and each is explained. Only then fix the order.
     - Evidence: the census in `bootstrap/step0-probes/` (4 codemods; 2 deliberate, 1 corruption,
       1 this hazard).
   - **0b batch 2 ✓ GREEN + pushed `1938294c7`**: 90 fixtures + 5 runes = 95, the ledger gone,
     SCOPE on all. The runes, repairs and 7 gaps are orchestrator-verified. **REFUTED → batch 2c**
     (`BRIEF-0b-REFUTE-batch-2c.md`):
     - 38 oracles appear nowhere in history, uncited (35 had history available);
     - fix = a per-fixture `ORACLE` file that the gate verifies against git;
     - also: the `to-faithful` coverage, 4 dead helpers, and `rename-wat-record`'s lying prose.
   - **0b batch 1 ✓ VERIFIED + pushed `8b2312dfb`** (16 chain fixtures, SCOPE on 30). Batch 2
     proceeds under `BRIEF-0b-ADDENDUM-batch-2.md`: close batch 1's 7 named coverage gaps first.
     **RULED 2026-09-12: delete `variant-vector-to-tagged-map`** (its `migrate` returns `src`), so
     the stems become 95.
   - **0b BRIEFED** (`BRIEF-0b-every-recorded-migration-replays.md`, `EXPECTATIONS-0b.md`):
     - **batch 1** is the 17 chain codemods plus SCOPE, then grok yields and the orchestrator
       verifies;
     - **batch 2** is the other 66, the 2 rotted `to-faithful-clojure` repaired, the
       unreadable-preimage runes (reproduced, not assumed), SCOPE on all 96, and the ledger deleted.
   - The probes are done: the tracked one in `scratch-pad/`, and end-to-end in `bootstrap/step0-probes/`.
2. The step script, plus a pilot on the first ~10 code commits: timed, with tricks catalogued.
3. Stone-sized batches with floor checkpoints. Grok executes; the orchestrator verifies.
4. Cross-check against `merge/grok-rete`, then floor and clippy, then main.

⚠ The post-merge TOOLING commits on this branch (A′ and (c) for `positional-ctor-to-map`, the
bootstrap record, these docs) are main-side work, not grok's. They must be carried onto the replay
branch. The chain runs over files grok touched, not all 2,072; runner guards are proven on real triggers.

## ⚠ RULINGS — do not re-litigate

- **Character case carries NO meaning.** `Enum.Variant` is a BIAS, never a rule.
- **A variant IS a tagged record; `Variant <: Enum`.** The edge was registered long ago
  (`types.rs:1135`); what was missing was `assignable`'s `Fn` arm (`58e9563b6`).
- **A keyword literal means two things** (data, or a variant ctor by registry lookup). Deferred ON
  PURPOSE to the symbol-head flip — `251-types-as-forms/NOTE-a-keyword-literal-means-two-things.md`.
- **`::` in names is the last EDN holdout.** Angle brackets and double-slash are ZERO live in code —
  every "live" hit I counted was inside a string literal.

## ⛔ THE FAILURE PATTERN OF THIS SESSION — five instances, one shape

Every grep-shaped instrument lied, and every correction came from a CONTROL:
26 "live" angle brackets (all strings) · 162 "lost" dispatch arms (main's `#[wat_intrinsic]` move;
`foldl` was on the list) · 533 "`::` files" (type names, not variants) · 10 "unbalanced" files
(parens inside strings; unbalanced BEFORE the chain too) · a `:grid::Result` census that stopped
at the first `])` — including on a type annotation. **Run the control first. Ask the substrate
(`--check`, `variant-parent-of`), not a regex.**

And one of the four-questions kind: I disqualified option A using "file A's answer can be wrong
for file B", then endorsed option B by calling that same fact a hidden defect. Two readings, one
fact, opposite verdicts — neither caught it because each option was weighed alone.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⛔ **YOU ARE ON `replay/grok-rete`, NOT main.** `merge/grok-rete` is the REFERENCE; its binary cannot
> boot. The frozen binaries live in `bootstrap/` (ignored), and their record is the `.gitignore` comment.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
