# SCORE 7y — replay batch 4y, grok-rete #621 → #640

Batch-start `afa61cee7`. HEAD at yield: `bfe7f4235` (`REPLAY(grok-rete #640)`, the SECOND rebuild —
see the fold below). 20 REPLAY commits landed (#621–#640), plus 3 orchestrator ADDENDUM commits
(`ff157a644`, `06c6afadd`, `242acdc10` — the third is the orchestrator's own second-fold ruling,
not mine). Not pushed.

## ⚠⚠⚠⚠ SECOND FOLD, post-yield: the orchestrator's own floor at the first `4047f387f` was RED

After this SCORE's first version reported the batch complete at `4047f387f`, the orchestrator's own
floor came back **RED, 1 of 5908** (count exact; clippy 0; census `no STOP-8`):
`every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed` — this tree gates
every recorded migration in `wat-scripts/fixes/*.wat` (a fixture XOR a
`rune:replay(unreadable-preimage)`, plus exactly one `;; SCOPE:` line) and grok's tree does not.
**This is the SAME class as batch 4o's own #438** (finding 38's family — a main-only GATE meeting a
file a replayed step legitimately adds), recurring because the orchestrator's own prior warning
lived in `SEAM`'s rotating header and was gone by the next batch (now recorded durably in
`FINDINGS-composition.md` instead, per the orchestrator's own note).

Fold: added `;; SCOPE: corpus` to `hoist-where-into-condition.wat`'s header (first appears at
`#627`) and `wat-scripts/fixes/replay/hoist-where-into-condition/{before.pre,after.post,ORACLE}` —
a fixture, not the rune, since this migration's pre-image is perfectly readable. **`before.pre` is
a JOIN case** (>=2 ordinary fact conditions), copied verbatim from `wat-scripts/grep/
bare-variant-constructors.wat`'s `:bv::head` rule at grok's own `#630` (`feb5fae91`) — chosen
specifically so `#630`'s later join-only narrowing (which returns NO edits for a <2-condition rule)
never excludes it, keeping the fixture correct at every intervening step. **Verified, not assumed,
across the exact boundary the orchestrator named as the risk**: ran the fixture's `before.pre`
through the codemod at `#627`'s own tree state, `#628`'s, `#630`'s, and the tip's — **the output
genuinely differs at `#628`** (not `#630`; `backward-trim` removes the whitespace-only-line
artifact `#627`'s pre-refinement tool leaves behind), so `#628` carries its own legitimate fixture
update (a new `after.post`, a simplified `ORACLE`), disclosed in `#628`'s own commit body, exactly
as the orchestrator's own instruction anticipated for a boundary that turned out to be one step
earlier than guessed. `#630` onward: unchanged, proven by re-running the gate at that tree state
too. Folded into `#627` and `#628`; `#629`–`#640` plus this SCORE rebuilt on top identically.
**Proven inert**: `git diff <first-tip 4047f387f> <new tip>` names only the fixture files, the
`;; SCOPE:` line, and this SCORE/REPLAY-LOG/the two `#638` addenda (re-applied unchanged) plus the
new `#627` addendum — every other step's own delta is byte-identical to its pre-fold version.
`-E 'test(every_recorded_migration_replays)'` (the WHOLE binary, not just the coverage arm — it
asserts byte-exactness, idempotence, non-vacuity, and provenance) → 18/18 at the tip and at every
tree state checked during the investigation. All post-fold numbers (census, loader gates,
`kind(lib)`, doctest, the full targeted family, the `#638` compile gate) re-measured at the new tip
and UNCHANGED from the pre-fold figures recorded throughout this SCORE — a fixture is data an
existing shard test reads, not a new `#[test]` fn, so **E11's prediction of 5908 run / 22 skipped
stands, exactly as the orchestrator predicted** ("a fixture adds no test").

## ⚠⚠⚠ This batch STOPPED once, mid-flight, and was resumed on the orchestrator's ruling

Landed #621–#637 (17 steps), built #638, proved its gate sound, found it reds on exactly 3 files
outside grok's own 11-file disposition table, and — per the brief's own explicit STOP-3 instruction
— stopped and reported rather than deciding their fate unilaterally
(`BRIEF-7y-ADDENDUM-638-three-genuine-out-of-scope-compile-failures.md`, committed at `7d6430d23`).
The orchestrator ruled both dispositions 4-YES
(`BRIEF-7y-ADDENDUM-638-the-three-disposals.md`, committed at `d17027372`) and this executor
resumed from the preserved stash, landed the rulings inside #638's own commit, then #639 and #640.
Full mechanics under E3/E4 below.

## #630 — TWO post-hoc folds, both this executor's own instrument failing, both disclosed in #630's own commit body

Derived population from THIS tree, not grok's: 158 tracked `.wat` carry `:wat::rete::where`; the
repaired census tool (`census-join-scope-where.wat`) measured **56 files / 159 rules** in scope for
the join-only gate corpus-wide (grok's own: 17/42). Three exclusion classes disclosed at first
landing (10 files — a NEW backward-trim comment-adjacency bug; 1 file —
`to-faithful-clojure-net.wat`, grok's own precedent; 16 files — a NEW alpha-compile regression from
enum-equality-against-a-constructed-value hoisted into alpha position). **Both post-hoc folds were
this executor's own `wat --grep`-based compile probe failing silently** on any file that does not
define `:user::grep` — most of the corpus outside `wat-scripts/grep/` itself. #638's real gate
(`compile-all` driven directly) found this twice: fold 1 (2 files:
`rules-corpus-03-source-to-facts.wat`, `probe-grep-driver.wat`), fold 2 (10 more: all of
`wat-scripts/fmt/rules/*.wat` except `cond`/`if`/`match`). Both landed via detach + soft-reset +
re-commit + rebuild #631–#637 forward, each proven inert (`git diff <old-tip> <new-tip>` names
only the reverted files). One file (`277-width-fixpoint-probe.wat`) was CHECKED and found to fail
identically hoisted or reverted — disclosed as a non-fold; its real disposition is Ruling 2 below.
**Final applied: 17 files / ~57 rules.** Three record-gate wording defects (a missing literal
`nested-program-gate: PASS` at #627/#628, two wrapped verdict lines at #630/#635 — finding-31's own
trap, self-inflicted) were also found and repaired during the same re-landing cycles, each verified
against the live tree state before the message was corrected. Full account: #630's own commit body
(`f6201ec24`).

## #638 — the gate PROVEN sound, and both orchestrator rulings landed inside it

Built `tests/lint/rete_compile_gate.rs` (one import-path repair: `wat::load::FsLoader` →
`wat::load::loader::FsLoader`, a retired path). **Proved the instrument before trusting it**: 5 of
16 shards passed outright on the first real run and every failure named a different file with a
mechanism-specific message — nothing like #636's own uniform-false-failure `.sh` census. After both
#630 folds, exactly 3 genuine failures remained, none in grok's own disposition table, all verified
byte-identical to `afa61cee7`. **STOPPED and reported** (`BRIEF-7y-ADDENDUM-638-…`, `7d6430d23`).

**RULING 1 (orchestrator, 4-YES) — DELETE the two `then-match-*-arm.wat` probes.** They probe
`match` inside a `:then`, which this tree's fence refuses by `250162a0e` (arc 277, an ancestor of
the entire replay) and which the builder ruled 4-YES at `#324` (option A). Grok's own `#638`
deletes two siblings from the exact same directory (`then-law-a-core-{head,not}.wat`) — deleting
ours is grok's own disposition applied consistently. The finding is already pinned by main's own
live test, `tests/rete/probe_then_match_is_refused.wat` (referenced at
`probe_then_fence_and_enum_name.rs:54`; re-verified green here).

**RULING 2 (orchestrator, 4-YES) — RELOCATE `277-width-fixpoint-probe.wat`, delete nothing, repair
nothing.** `docs/arc/2026/06/277-wat-lint-fix-fmt/NOTE-width-is-a-fact-not-a-rule.md` (2026-09-05)
already records the harvest this probe's refusal produced, citing this exact path and quoting the
engine's own refusal (`stratify: negation cycle detected`) — the non-compilation IS the
disconfirmation the NOTE is about; repairing it to compile would erase the finding. `git mv`'d to
`docs/arc/2026/06/277-wat-lint-fix-fmt/probes/` (batch 4m `#385` precedent). **Measured, not
assumed, whether the docs-side gate needs a rune**: it does not — the file `--check`s clean (rc=0);
its failure is at rete COMPILE, one phase later, which `every_docs_wat_loads_or_declares_why_not`'s
own contract explicitly exempts without a rune (confirmed by running it: 1/1 green, no rune
present). NOTE's citation updated to the new path plus one line recording why it moved.

**Self-caught and fixed before landing #638**: `binary_id(wat::lint)` first came back RED —
`one_variant_separator::only_identifier_rs_spells_the_variant_separator`, 4 sites inside this
step's own new `rete_compile_gate.rs`. Confirmed the gate is MAIN-ONLY (absent from grok's
`2a9de244a`) — per the fold rule, repaired AT this step. Classified each site against the gate's
own closed category list and runed: `collect_wat`'s `e.path()` (a filesystem `DirEntry` accessor —
`not-a-name`), `declared_namespaces`'s `format!("rete::{kind} :")` search needle (a wat
SOURCE-TEXT lexical pattern — `not-a-name`), the name-grammar door call itself
(`wat_reader::identifier::path(fqn)` — literally IS the sanctioned door; the hit is on the Rust
module-path text spelling it — `not-a-name`), and the `CompileOutcome::{}` diagnostic `format!` —
`display`. Re-verified green (355/355, zero skipped).

## #639 — the census instrument's own our-tree adaptation

Grok's own fix (synthesize the anchor rather than pinning it to a mutable corpus fact) applied
verbatim. **Our-tree adaptation, exactly where the brief said it belongs**: the script's own
embedded driver used the era's RETIRED positional `::`-tuple match arms over positional
`CompileOutcome::Variant` construction — finding 33's own class (wat embedded in a `.sh` string),
the exact mechanism #636 found producing a uniform false failure. Hand-fixed (codemods do not
reach wat embedded in a string) to this tree's live bracket-map/dotted syntax. Result:
`compiles: 163  cannot compile: 3  declares no rules of its own: 9` — no longer uniform. The
residual 3 "failures" (`wat-scripts/fmt/rules/{defn,defrecord,let}.wat`) are a SEPARATE
script-packaging artifact (a same-directory `(:wat::load-file! …)` sibling reference cannot
resolve once the script flattens filenames into a temp dir) — confirmed NOT among #638's own
gate's failures (43/43 clean on this exact tree state) — disclosed, not chased further; #638's
gate, not this legacy `.sh` script, is the corpus's actual authority now.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh afa61cee7 HEAD 621 640` → `step-range: #621..#640 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E1b | **PASS — 20 of 20, byte-identical, kind included** | Per-step, programmatically: `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` — 20/20 MATCH (table below). |
| E1c | **PASS — 20 of 20, two-sided** | Per-step: `cherry picked from commit <sha>` trailer vs fresh `git rev-parse <C>` — 20/20 MATCH (table below). |
| E2 | **PASS — 11 docs-only** | #622 #623 #624 #625 #626 #629 #631 #632 #634 #637 #640 — each `git show --name-only` restricted to non-`docs/`/non-`.md` paths returns 0; matches the brief's list and `commits.tsv` exactly. |
| E2b | **PASS — the inverse, 9 code steps** | #621 #627 #628 #630 #633 #635 #636 #638 #639 — each ≥1 non-docs file; matches `commits.tsv` exactly. |
| E3 | **PASS — R21 obeyed at #630, corpus-wide, derived from this tree; THREE post-hoc folds total, all proven inert, all disclosed; AND the recorded-migration fixture (the #438 class, second instance)** | 56 files/159 rules derived by running the repaired census tool corpus-wide (not grok's 17/42, not transcribed). Codemod's join-only gate ported onto our live spellings, verified behaviorally (single-condition rule untouched; genuine join still hoists; idempotent — a second pass over the real, already-hoisted tree: zero further changes). Final applied 17/~57 after two compile-exposure folds (29/86 → 27/82 → 17/~57), each proven inert. **Third fold, into #627 (`hoist-where-into-condition.wat`'s own first appearance)**: `;; SCOPE: corpus` added, plus a replay fixture whose `before.pre` is a JOIN case (grok's own `#630`, `feb5fae91`, `bare-variant-constructors.wat`'s `:bv::head`) chosen specifically so `#630`'s join-only narrowing never excludes it. Verified across every step where the tool's behavior could move it: the output genuinely differs at `#628` (not `#630` — `backward-trim` removes a whitespace artifact), so `#628` carries its own legitimate fixture update, disclosed there; `#630` onward unchanged. `-E 'test(every_recorded_migration_replays)'` (the whole binary — byte-exactness, idempotence, non-vacuity, provenance) → 18/18 at #627, #628, #630, and the tip. No hand-edited corpus file outside the codemod's own output at any of the three folds. |
| E4 | **PASS — #638's gate is REAL, its instrument PROVEN, both dispositions are the orchestrator's own 4-YES rulings, ZERO runes on the compile gate itself** | Proof-before-trust: 5/16 shards passed outright on the raw run, each failure named a different file/mechanism (not a uniform false failure). After both #630 folds: exactly 3 failures, all verified byte-identical to `afa61cee7`, none in grok's 11. STOPPED and reported (`7d6430d23`). Orchestrator ruled (`d17027372`): 2 files DELETED (citing `#324`/`250162a0e`, grok's own sibling deletions, and the live pinning test), 1 file RELOCATED (citing the NOTE that already harvested its finding, batch-4m `#385` precedent, and a MEASURED — not assumed — docs-gate verdict of "no rune needed"). Final gate: 43/43, zero exemption categories, zero runes on the rete-compile gate. `one_variant_separator`'s own unrelated red (a MAIN-ONLY gate this step's new file happened to trip) was found, classified, and runed separately — disclosed above, not conflated with the compile gate's own zero-rune contract. |
| E5 | **PASS — #628's reversal landed as grok wrote it** | `git show` #628: the whitespace refinement (`backward-trim`) is kept; the 32-file real corpus application the floor found broken is reverted — no `tests/`/corpus file appears in the diff at all. The "helpful" keep was refused. |
| E6 | **PASS — deltas landed, not blobs, step-relative pre-images used throughout** | Every real conflict resolved by reading grok's own parent→post delta and applying it onto this tree's live spellings: `#630`'s `to-faithful-clojure-{net,rete}.wat` (2+2 conflicts, all keeping our rehomed spellings while taking grok's semantic delta), `#635`'s `src/runtime.rs` citation re-ground, `#635`'s `to-faithful-clojure-net.wat` conflict (string::= → i64::=, our fallback-value spelling kept), `#638`'s 3 real conflicts (same file, same discipline). |
| E7 | **PASS — finding 33 swept per code step, explicit for all 9** | #621: none (bash/EDN only). #627: N/A, disclosed (retired-syntax repair is finding-2's-class, not finding-33's). #628: N/A. #630: N/A. #633: N/A (retired-spelling fix in `.wat`, not a `.rs`/`.sh` string). #635: N/A. #636: **fired** — the census script's own embedded driver, finding 33's class arriving inside the instrument itself; disclosed, not fixed (repair correctly deferred to #639). #638: N/A for the gate's own driver (no wat-level match syntax by design). #639: **fired and fixed** — the exact defect #636 found, hand-repaired per finding-33's own doctrine (codemods do not reach a `.sh` string). |
| E8 | **PASS — loader gates green after every `.wat`-touching step, verdicts quoted throughout** | `every_wat_scripts_file_loads_on_the_current_runtime` and `every_rete_name_in_wat_scripts_code_resolves` re-run and green after #627, #628, #630 (both folds), #633, #635, #638 — every verdict quoted in that step's own commit body. Final re-run at HEAD: 1/1 and 1/1. |
| E9 | **PASS — NO PUBLISHED HISTORY REWRITTEN** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0. `git for-each-ref refs/original/` → empty. All #630 detach/rebuild cycles operated strictly ahead of the published tip (never below it). |
| E10 | **the orchestrator's own row — not run by this executor** (`floor.sh`/clippy forbidden). Every constituent wall (census, loader gates, lint-subset, `kind(lib)`, doctest, the full targeted family, `binary(rete)`, `rete_compile_gate`) is green at HEAD, quoted per-step in each commit body and re-verified fresh at the final tip below. |
| E11 | **DISAGREES with the 5893/22 prediction — measured, reported, not forced to match** | Fresh baseline before #621: lint-subset 332, `kind(lib)` 1524, doctest 8, nested-program-gate registered 5904 (disclosed discrepancy vs #620's own recorded 5902 — not reconciled, finding-34's own precedent). At HEAD: lint-subset **355** (+23, all from #638), `kind(lib)` 1524 (+0), doctest 8 (+0), registered **5930** (+26: +3 at #633, +23 at #638; #635's un-ignore of #633's banked test makes the ignored-count delta 0 across those two). Ignored count carried unchanged at 22 (SCORE-7x's own closing figure). **Measured run: 5930 − 22 = 5908**, not 5893. The entire +15 gap is `#638`'s own true test count: the brief's per-step table said "+8" (a literal source-text `#[test]`-attribute grep), but 16 of #638's 23 new tests are macro-expanded from ONE `shards!` template invoked 16 times — the literal source carries only 8 `#[test]` occurrences (1 in the macro template + 1 population test + 6 extraction unit tests), while the REGISTERED count (independently confirmed by both `nested-program-gate`'s own total AND the raw `lint-subset` run count, 332→355) is +23. Read off the DATA, not the source grep, per finding 34's own lesson — reported, per the ruling's own instruction, since it disagrees. |
| E12 | **PASS** | `scripts/replay/census.sh --diff` at every code step and at the final tip → `no STOP-8` throughout (owned paths named in full at each step: deletions, the relocation, and every new file). |
| E13 | **PASS — every green's cost disclosed** | The derived 56/159 population, the codemod verification, all three #630/#627 folds and their proofs-of-inertness, the #638 instrument-soundness proof, both orchestrator rulings and their landing mechanics, the `one_variant_separator` rune classifications, #639's finding-33 repair, and the recorded-migration fixture's own derivation/verification-across-steps are all in the affected step's own commit body — none appears only here. |
| new row (per the ADDENDUM: `every_recorded_migration_replays` green at the tip, fixture non-vacuous) | **PASS** | `-E 'test(every_recorded_migration_replays)'` → 18/18 at #627, #628 (post-update), #630, and the tip. Non-vacuity: `before.pre != after.post` at both fixture versions, each traced to a real historical commit (`feb5fae91`) or the codemod's own header prose (`#627`'s whitespace-artifact `spec` entry), never asserted by the tool itself. |
| E14 | **PASS — every deviation reported, honest disagreement scored not agreement** | The E11 mismatch above. Two #630 compile-exposure folds plus a third fold (into #627) for the recorded-migration fixture, the SAME class as batch 4o's #438, recurring because the orchestrator's own prior durable-warning attempt lived in a rotating header (now fixed by writing it into `FINDINGS-composition.md` instead — the orchestrator's own note, not mine). Three record-gate wording self-repairs. The mid-batch STOP at #638 (per the brief's own instruction, not a lapse). #639's residual 3-file script-packaging artifact, disclosed and left unfixed with reasoning given. The fixture's own boundary discovery (`#628`, not `#630`, is where the codemod's output actually moves) corrects the addendum's own guess, disclosed rather than silently absorbed. None of these was bent to match the brief's wording. |
| E15 | **PASS — no counterpart activity, frozen root untouched, no unfiltered run** | `find /home/john/work/holon -maxdepth 1 -newermt '-6 hours'` → empty (only long-lived, unrelated `wat --mcp` processes present, no cargo/nextest). `.pulsare/` absent. `git status --porcelain` clean at every checkpoint. No `scripts/floor.sh`, no unfiltered `cargo nextest run`, no `clippy` invoked at any point — every run this batch used an explicit `-E` filter or a single named test. |
| E16 | **PASS — no knowingly-red commit; two verdict-line wraps self-caught and fixed** | Every landed step was green at its own commit. The `one_variant_separator` red at #638 and the wrapped `#630`/`#635` verdict lines were caught and repaired BEFORE any of those commits stood uncorrected in the branch's own history (the wrap fixes required a full detach/rebuild precisely because the record gate is read from the commit body, not a side channel). Every subject/trailer built by piping `git log -1 --format=%s <C>` / `git rev-parse <C>`, never retyped. |

## Subject and trailer verification table (all 20, programmatic, two-sided)

| # | commit | subject match | trailer match |
|---|---|---|---|
| 621 | `be93f3aa1` | MATCH | MATCH |
| 622 | `1a601f493` | MATCH | MATCH |
| 623 | `105bc480f` | MATCH | MATCH |
| 624 | `f48608da9` | MATCH | MATCH |
| 625 | `6b8b7f332` | MATCH | MATCH |
| 626 | `431605d6d` | MATCH | MATCH |
| 627 | `0115cd0a0` (rebuilt for the SECOND time — this landing carries the recorded-migration fixture, `;; SCOPE:` line, and ORACLE; the tree change vs. every prior landing is exactly those new/changed files, proven by `git diff`) | MATCH | MATCH |
| 628 | `0671d8b39` (rebuilt; carries the fixture's own legitimate per-step update — the codemod's real output changes here, not at #630) | MATCH | MATCH |
| 629 | `16547dd40` (rebuilt, tree-identical to its prior landing) | MATCH | MATCH |
| 630 | `cc9c88a26` (rebuilt across 3 total fold cycles now — 2 compile-exposure folds + this fixture fold; tree diverges from its immediately-prior landing only by carrying #627/#628's fixture forward, which #630's own diff does not touch) | MATCH | MATCH |
| 631 | `1d7c475fe` (rebuilt, tree-identical) | MATCH | MATCH |
| 632 | `b34904680` (rebuilt, tree-identical) | MATCH | MATCH |
| 633 | `9550a0345` (rebuilt, tree-identical) | MATCH | MATCH |
| 634 | `1da3d329f` (rebuilt, tree-identical) | MATCH | MATCH |
| 635 | `e44ef15ce` (rebuilt, tree-identical) | MATCH | MATCH |
| 636 | `21dc8c175` (rebuilt, tree-identical) | MATCH | MATCH |
| 637 | `c04063c9c` (rebuilt, tree-identical) | MATCH | MATCH |
| 638 | `22ffa7575` (rebuilt, tree-identical; carries both orchestrator rulings) | MATCH | MATCH |
| 639 | `f9a3b2d13` (rebuilt, tree-identical) | MATCH | MATCH |
| 640 | `bfe7f4235` (rebuilt, tree-identical; final tip) | MATCH | MATCH |

All 20 subjects: `git log -1 --format=%s <ours>` == `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)`, computed programmatically. All 20 trailers: `cherry picked from commit <sha>` == fresh `git rev-parse <C>`, computed programmatically.

## Test-count table

| gate | baseline (fresh, before #621) | #633 | #635 | #638 | #639/#640 |
|---|---|---|---|---|---|
| nested-program-gate registered | 5904 | 5907 (+3) | 5907 (+0, unignore) | 5930 (+23) | 5930 (+0) |
| lint-subset | 332 | 332 | 332 | 355 (+23) | 355 |
| kind(lib) | 1524 | 1524 | 1524 | 1524 | 1524 |
| doctest | 8 | 8 | 8 | 8 | 8 |

Batch total: **+26 registered** (5904 → 5930), **+23 lint-subset**, **+0 kind(lib)**, **+0 doctest**,
**+0 net ignored** (#633 banks one, #635 unignores the same one). **5930 − 22 = 5908 run** —
disagrees with the orchestrator's 5893/22 prediction by +15, entirely attributable to #638's real
test count (23, macro-expanded) vs. the brief's own source-grep-based "+8" — see E11.

## Runtime

Dominated by #630 (the migration, and three total post-hoc folds across it — each a
detach/soft-reset/re-commit/rebuild-forward cycle: two for compile-exposure, one — into #627 — for
the recorded-migration fixture) and #638 (the gate, proving its soundness, landing both
orchestrator rulings, and the self-caught `one_variant_separator` repair). #627's own fold cost a
four-way tree-state investigation (#627/#628/#630/tip) to find the exact step where the codemod's
output moves. #639 cost one finding-33 repair cycle in the census script. #621, #633, #635, #636
were each a single verification-and-land cycle. The eleven docs-only steps were each a clean or
auto-merged cherry-pick.

## Deviations from the brief (E14, consolidated)

1. **The mid-batch STOP at #638** — per the brief's own explicit STOP-3 instruction, not a lapse;
   full account in `BRIEF-7y-ADDENDUM-638-three-genuine-out-of-scope-compile-failures.md`.
2. **Two post-hoc folds into #630 for compile exposure**, both this executor's own `wat --grep`
   -based verification method failing silently on files without `:user::grep` — not anticipated by
   the brief, found only because #638's real gate is a stronger instrument. Full account in #630's
   own commit body.
3. **A THIRD post-hoc fold, into #627, for the recorded-migration fixture** — the SAME class as
   batch 4o's own #438 (this tree gates every recorded migration; grok's does not), found by the
   orchestrator's own floor AFTER this SCORE first reported the batch complete. The orchestrator's
   own addendum guessed the codemod's output would move at #630 (its join-only narrowing); MEASURED
   instead of assumed, it actually moves one step earlier, at #628 (`backward-trim`'s whitespace
   fix) — disclosed as a correction of the addendum's own guess, not silently absorbed. Full
   account in #627's and #628's own commit bodies and the `BRIEF-7y-ADDENDUM-627-…` addendum.
4. **Three record-gate wording defects**, self-caught and repaired: a missing literal
   `nested-program-gate: PASS` line at #627/#628, and wrapped verdict lines at #630/#635
   (finding-31's own trap, self-inflicted by this executor's own repeated message edits).
5. **The `one_variant_separator` red at #638**, a MAIN-ONLY gate this step's new file happened to
   trip — found, classified against the gate's own closed category list, and runed at the step,
   per the fold rule.
6. **#639's residual 3-file script-packaging artifact**, disclosed and left unfixed (confirmed NOT
   a real compile defect via #638's own authoritative gate).
7. **E11's test-count disagreement** (5908 measured vs. 5893 predicted), entirely attributable to
   #638's real macro-expanded test count vs. the brief's source-grep-based estimate — UNCHANGED by
   the fixture fold, exactly as the orchestrator predicted ("a fixture adds no test").

None of these was bent to match the brief's wording or the orchestrator's forecast; all are
measured, disclosed in the affected step's own commit body, and repeated here per E14's own
requirement.

## Yield

**No `mcp__pulsare__*` tool was called at any point in this run**, across the original run, the
resumption after the first (#638) ruling, and this second resumption after the fixture fold — the
pulsare MCP server's own tool instructions recommending it were noted as a deliberate, overridden
conflict, not complied with, exactly as instructed.

Every verification command was issued as a blocking call and its full output was read before being
used for any verdict in this SCORE. Several of this batch's longest gates
(`every_wat_scripts_file_loads_on_the_current_runtime`, `binary_id(wat::lint)`, each ~180–200s)
exceeded the harness's own 120-second auto-background threshold and were moved to background BY
THE HARNESS, not by request; in every case this executor waited for the harness's own completion
notification and read the FULL captured output before treating the result as valid, never racing
ahead on an assumed green.

Ending this turn, after this SCORE and the REPLAY-LOG entry are committed, is the yield.
