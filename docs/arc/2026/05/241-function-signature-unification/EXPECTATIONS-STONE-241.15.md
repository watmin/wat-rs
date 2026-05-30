# EXPECTATIONS — Stone 241.15 — ZOMBIE PURGE (Wipe-the-board stone)

Independent scorecard. NO vigilia required (no namespaced home). SCORE-green commit. Upper bound 120 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-06 PASS 6/6 | `cargo test --release --test probe_arc241_stone15_zombie_purge` | 6/0 |
| 2 | Stone 241.14 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone14_restricted_absorbed` | 6/0 |
| 3 | Stone 241.13 probe preserved 2/2 | `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut` | 2/0 |
| 4 | Stone 241.12 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone12_defalias` | 5/0 |
| 5 | Stone 242.x + 241.11 + 241.10 probes preserved | each | counts preserved |
| 6 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 8 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 930 |
| 9 | Pre-INSCRIPTION grep CLEAN (`:wat::core::try`) | `grep -rn ":wat::core::try\b" src/ tests/ wat/` | 0 active matches |
| 10 | Pre-INSCRIPTION grep CLEAN (lowercase expect forms) | `grep -rn ":wat::core::option::expect\b\|:wat::core::result::expect\b" src/ tests/ wat/` | 0 active matches |
| 11 | RETIREMENT_TABLE has 12 entries | `grep -cE '^    \(":wat' src/remedy/retirement.rs` | 12 entries |
| 12 | SCORE-STONE-241.15.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.15.md` | file exists |

## Structural verification (10 rows)

| Verification | Command | Expected |
|---|---|---|
| 3 HARD-CUT arms in check.rs | `grep -n '":wat::core::try"\|":wat::core::option::expect"\|":wat::core::result::expect"' src/check.rs \| grep -c "Stone 241.15"` | ≥ 3 (one arm per zombie) |
| 10th/11th/12th RETIREMENT_TABLE entries verbatim | `grep -A2 'Stone 241.15' src/remedy/retirement.rs` | matches `(":wat::core::try", ":wat::core::Result/try")` + `(":wat::core::option::expect", ":wat::core::Option/expect")` + `(":wat::core::result::expect", ":wat::core::Result/expect")` |
| Dispatch arms deleted in runtime.rs | `grep -n '":wat::core::try" => eval_try' src/runtime.rs` | 0 matches |
| Lowercase dispatch arms deleted in runtime.rs | `grep -n '":wat::core::option::expect" => eval_option_expect\|":wat::core::result::expect" => eval_result_expect' src/runtime.rs` | 0 matches |
| Canonical dispatch arms preserved | `grep -n '":wat::core::Result/try" => eval_try\|":wat::core::Option/expect" => eval_option_expect\|":wat::core::Result/expect" => eval_result_expect' src/runtime.rs` | 3 matches (one per canonical) |
| Eval functions UNCHANGED | `grep -n 'fn eval_try\|fn eval_option_expect\|fn eval_result_expect' src/runtime.rs` | 3 matches (functions still defined) |
| Soft-deprecation helper functions DELETED | `grep -n 'fn .*deprecat.*try\|fn .*deprecat.*expect' src/check.rs` | 0 matches |
| special_forms.rs entries deleted (3 zombies) | `grep -n '":wat::core::try"\|":wat::core::option::expect"\|":wat::core::result::expect"' src/special_forms.rs` | 0 matches |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| Doc migration touched expected files | `git diff --name-only docs/USER-GUIDE.md docs/SERVICE-PROGRAMS.md docs/CLOJURE-ROSETTA.md docs/WAT-CHEATSHEET.md` | 4 files modified |

## Prediction: 60–120 min Mode A

Stone 241.15 scope decomposition:
- 3 HARD-CUT arms + 3 RETIREMENT_TABLE entries — **~15 min**
- Dispatch arm deletions (runtime.rs 3 arms + check.rs 1 standalone + 2 surgical removals) — **~15-20 min**
- Soft-deprecation helper function deletions (3 functions + callers) — **~10-15 min**
- special_forms.rs entry deletions (3-4 entries) — **~5 min**
- Reflection emitter audit (likely zero work) — **~5 min**
- Doc cascade (USER-GUIDE + SERVICE-PROGRAMS + CLOJURE-ROSETTA + WAT-CHEATSHEET; bulk sed per pattern) — **~15-20 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10 min**

Within-band: 60-120 min. Under-band likely (apparatus is mature; bulk-sed-friendly doc patterns; no architectural unknowns).

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `4bb6bbbe`)

1. Stone 241.15 probe at HEAD: **0/6 PASS** (all 6 contracts disconfirm cleanly via Stone 241.15 marker absence)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. Probe 241.14: 6/6 · 241.13: 2/2 · 241.12: 5/5 · 242.2: 6/6 · 242.1: 4/4 · 241.11: 5/5 · 241.10: 8/8 · prior probes preserved
4. Clippy: **906** (post-Stone-241.14; gate raised to ≤ 930 for substrate-refactor line-shift)
5. RETIREMENT_TABLE: **9 entries** (will grow to 12 — Stone 241.15 S2)
6. Active zombie surface:
   - `:wat::core::try`: 0 wat callers + 0 test callers + 7 doc sites + ~6 substrate sites
   - `:wat::core::option::expect`: 0 wat/test + 11 doc sites + ~3 substrate sites
   - `:wat::core::result::expect`: 0 wat/test + 1 doc site + ~3 substrate sites

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Eval/infer functions accidentally damaged when dispatch arms deleted | canonical-form probes regress | strict surgical deletion of dispatch arms only; eval functions untouched |
| **T2** | Dispatcher routing helpers (check.rs:2703-2734, 2823-2839) require surgical OR-clause removal | per-line audit | remove `|| head_str == retired` clauses; canonical-routing stays |
| **T3** | special_forms.rs:349 unclear context | sonnet reads context | judge DELETE vs KEEP based on surrounding code |
| **T4** | Doc bulk sed could match unintended sites | per-pattern verification | confirm no overlap between `:wat::core::try` and `:wat::core::Result/try` strings before bulk sed |
| **T5** | Reflection emitters for retired forms (Stone 241.12/13/14 trap-door class) | grep audit | likely zero; migrate if found |
| **T6** | "Soft deprecation served help table" temptation | self-audit | STOP per `feedback_hard_cut_admits_no_bypasses`; the user direction is "wipe the board" |
| **T7** | Auto-fixer temptation (doc cascade scale) | post-strike `ls crates/` | acceptable if EPHEMERAL — built, used, DELETED |
| **T8** | Sonnet drafts INTERSTITIAL | post-strike grep | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T9** | SCORE doc not written | post-strike `ls SCORE-STONE-241.15.md` | DISCIPLINE GAP per `feedback_score_present_check_before_closure` |
| **T10** | Stone 241.16 scope creep | post-strike grep for is_mutation_head/parse_define_form touches | STOP; revert |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally
- 10/10 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.15.md`

### Phase B — NOT cast (no vigilia)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/check.rs` (3 HARD-CUT arms + soft-deprecation helper deletion + dispatch surgical edits), `src/runtime.rs` (3 dispatch arm deletions), `src/special_forms.rs` (3-4 entry deletions), `src/remedy/retirement.rs` (3 entries), `src/closure_extract.rs` (if reflection emitters touched), docs/USER-GUIDE.md, docs/SERVICE-PROGRAMS.md, docs/CLOJURE-ROSETTA.md, docs/WAT-CHEATSHEET.md, SCORE doc
- INTERSTITIAL NOT in commit (orchestrator authors after Stone 241.17 INSCRIPTION)
- Push to origin
- Stone 241.16 (Enemy 3 — define eval-time residue) opens next; DESIGN drafted after 241.15 ships

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.12 | defalias mint native + 13-caller cascade + S6 consistency pass | +622/-229 | 60-150 min | ~130 min + context boundary |
| 241.13 | define-dispatch HARD CUT + 445-line file DELETED + plumbing cascade + 6 test files | +340/-1203 | 90-180 min | ~25 min (substantially under-band) |
| 241.14 | def-restricted + defn-restricted absorption + storage deletion + walker rewrite | +768/-739 | 90-180 min | ~26 min (substantially under-band) |
| **241.15 (this)** | **3 zombies HARD CUT + dispatch arm deletions + soft-deprecation helper deletions + doc cascade** | **TBD (probably +50/-200 net)** | **60-120 min** | **TBD** |

## What this unblocks

**Stone 241.16** — Enemy 3 (`:wat::core::define` eval-time residue completion). With the board wiped, Enemy 3 gets focused attention.

**Stone 241.17** — INSCRIPTION closes arc 241 + `feedback_defer_by_naming` doctrine memory inscribed.

**Arc 237.8b** — reopens after Stone 241.17 per `feedback_no_regression_until_arc_done`

**One-canonical-path doctrine** — three more violations annihilated. `:wat::core::Result/try` is THE try form. `:wat::core::Option/expect` + `:wat::core::Result/expect` are THE expect forms. PascalCase Type/method canonical; lowercase-namespace duplicates DEAD; arc-109 retired form actually HARD CUT (not just labeled).
