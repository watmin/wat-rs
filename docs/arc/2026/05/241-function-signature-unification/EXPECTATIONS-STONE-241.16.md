# EXPECTATIONS — Stone 241.16 — `:wat::core::define` EVAL-TIME RESIDUE COMPLETION (Enemy 3 of 4)

Independent scorecard. NO vigilia required. SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-04 PASS 4/4 | `cargo test --release --test probe_arc241_stone16_define_eval_residue` | 4/0 |
| 2 | Stone 241.15 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone15_zombie_purge` | 6/0 |
| 3 | Stone 241.14 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone14_restricted_absorbed` | 6/0 |
| 4 | Stone 241.13 probe preserved 2/2 | `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut` | 2/0 |
| 5 | Stone 241.12 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone12_defalias` | 5/0 |
| 6 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 7 | Stone 241.10 + arc 242 + arc 237/238 probes preserved | each | counts preserved |
| 8 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL (test migrations may shift; document) |
| 9 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 10 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 935 |
| 11 | Pre-INSCRIPTION grep CLEAN | `grep -rn ":wat::core::define\b" src/ tests/ wat/` | 0 active uses outside acceptable categories per S12 |
| 12 | SCORE-STONE-241.16.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.16.md` | file exists |

## Structural verification (10 rows)

| Verification | Command | Expected |
|---|---|---|
| `parse_define_form` function DELETED | `grep -n "fn parse_define_form" src/runtime.rs` | 0 matches |
| `is_define_form` function DELETED | `grep -n "fn is_define_form" src/runtime.rs` | 0 matches |
| `:wat::core::define` absent from `is_mutation_form` | `grep -A20 "fn is_mutation_form" src/freeze.rs \| grep "wat::core::define"` | 0 matches |
| `:wat::core::define` absent from `is_declaration_form` | `grep -A20 "fn is_declaration_form" src/freeze.rs \| grep "wat::core::define"` | 0 matches |
| `:wat::core::define` absent from `is_mutation_head` | `grep -A20 "fn is_mutation_head" src/runtime.rs \| grep "wat::core::define"` | 0 matches |
| special_forms.rs entry DELETED | `grep -n "insert.*:wat::core::define" src/special_forms.rs \| grep -v "define-dispatch\|defstruct\|defenum\|defalias\|defmacro\|defn" \| grep '":wat::core::define"'` | 0 matches |
| special_forms.rs audited-forms test entry DELETED | `grep -n '":wat::core::define",' src/special_forms.rs \| grep -v "defstruct\|defmacro\|defalias\|defn"` | 0 matches |
| Stone 241.16 HARD-CUT marker present | `grep -n "Stone 241.16" src/check.rs` | ≥ 1 match (the HARD-CUT-rejection arm carrying the marker) |
| Bypass-rejection tests migrated | `grep -n ":wat::core::define" src/freeze.rs` | only historical comments + Stone 241.16 deletion markers (active use migrated to defstruct or similar) |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |

## Prediction: 90–180 min Mode A

Stone 241.16 scope decomposition:
- `parse_define_form` deletion + ~30 error-construction site cascade — **~30-45 min**
- `is_define_form` + caller deletion — **~5 min**
- Form-predicate arm deletions (3 functions × 1 arm each) — **~10 min**
- check.rs processing arm deletions + check.rs:7049 investigation — **~15-20 min**
- special_forms.rs entries (2) deletion — **~5 min**
- Error message string migration (3 sites) — **~5-10 min**
- Test fixture migration (bypass-rejection × 3 + wat_arc144 × 2 + probe_let/do_splice_define judgment) — **~30-45 min**
- Reflection emitter audit — **~5 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 90-180 min. Stone 241.16 is comparable in scope to Stone 241.13 (445-line file deletion + plumbing cascade). Recent stones (241.13/14/15) all under-band; this one is BIGGER + has substantive test migration so closer to middle-band.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `8a0c536e`)

1. Stone 241.16 probe at HEAD: **3/4 PASS** (C01 disconfirms — Stone 241.16 marker absent; C02-C04 preservation — Stone 241.11 HARD CUT still fires)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. All prior probes preserved
4. Clippy: **906** (post-Stone-241.15; gate raised to ≤ 935 for substrate refactor)
5. `parse_define_form` exists at runtime.rs:4399+ (~30 error-construction sites within)
6. `is_define_form` exists at runtime.rs:3547
7. `is_mutation_head` arm at runtime.rs:27427 includes define
8. `is_mutation_form` arm at freeze.rs:1312 includes define
9. `is_declaration_form` arm at freeze.rs:1355 includes define
10. special_forms.rs:175 entry registered; line 331 in audited-forms test
11. Bypass-rejection tests at freeze.rs:1651/1807/1985 use `:wat::core::define` as fixture
12. wat_arc144_special_forms.rs:210-211 asserts define IS special form (will fail post-stone)
13. wat_arc144_uniform_reflection.rs:103/121 asserts reflection emits define (already STALE post-Stone-241.12 reflection migrations)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | parse_define_form deletion cascades through ~30 error-construction sites | compile errors | per substrate-as-teacher; mechanical |
| **T2** | `register_defines` function name preservation question (D4) | sonnet judgment | preserve name unless rename cascade is < 10 sites |
| **T3** | check.rs:7049 arm context — KEEP vs DELETE judgment | per BRIEF S5 | sonnet reads context; either path produces probe-passing behavior |
| **T4** | Bypass-rejection test fixture migration — chosen alternative head must be REGISTERED but not in the test's specific bypass | per-test review | recommend defstruct (clearly registered; bypass scenario different from instance) |
| **T5** | Reflection emitter for :wat::core::define (Stone 241.12/13/14/15 trap-door class) | grep audit | likely zero; migrate if found |
| **T6** | Test migration may surface other test files referencing define in fixtures | full grep | per-file audit; migrate per-fixture or delete pre-Stone-241.11 tests |
| **T7** | "Defense-in-depth preservation" sonnet temptation | self-audit | STOP per `feedback_hard_cut_admits_no_bypasses` |
| **T8** | Auto-fixer temptation (cascade scale) | post-strike `ls crates/` | acceptable if EPHEMERAL — built, used, DELETED |
| **T9** | Sonnet drafts INTERSTITIAL | post-strike grep | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T10** | SCORE doc not written | post-strike `ls SCORE-STONE-241.16.md` | DISCIPLINE GAP per `feedback_score_present_check_before_closure` |
| **T11** | Stone 241.17 scope creep — sonnet drafts INSCRIPTION | post-strike grep for INSCRIPTION-related changes | STOP per D8; INSCRIPTION is orchestrator-direct |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally
- 10/10 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.16.md`

### Phase B — NOT cast (no vigilia)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/runtime.rs` (massive parse_define_form deletion + is_define_form + is_mutation_head arm + error message migrations), `src/check.rs` (processing arm deletions + check.rs:7049 disposition + error message migrations), `src/freeze.rs` (is_mutation_form/is_declaration_form arms + bypass-rejection test fixture migrations), `src/special_forms.rs` (entries deleted), `src/closure_extract.rs` (if reflection emitters touched), test files per S8 inventory, SCORE doc
- INTERSTITIAL NOT in commit (D6; orchestrator authors after Stone 241.17 INSCRIPTION closes arc 241)
- Push to origin
- **Stone 241.17 (INSCRIPTION; orchestrator-direct paperwork) opens next**

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.12 | defalias mint native + 13-caller cascade + S6 consistency pass | +622/-229 | 60-150 min | ~130 min + context boundary |
| 241.13 | define-dispatch HARD CUT + src/dispatch.rs DELETED (445 lines) + plumbing cascade + 6 test files | +340/-1203 | 90-180 min | ~25 min (substantially under-band) |
| 241.14 | def-restricted + defn-restricted absorption + storage deletion + walker rewrite | +768/-739 | 90-180 min | ~26 min (substantially under-band) |
| 241.15 | 3 zombies HARD CUT + dispatch arm deletions + soft-deprecation helper deletions + doc cascade | +329/-200 | 60-120 min | ~8.7 min (substantially under-band; THIRD in a row) |
| **241.16 (this)** | **parse_define_form DELETED (~30 sites) + is_define_form DELETED + form-predicate arms removed + check.rs processing arms + special_forms entries + error message migrations + 5-6 test fixture migrations** | **TBD (probably +50/-400 net)** | **90-180 min** | **TBD** |

## What this unblocks

**Stone 241.17 — INSCRIPTION closes arc 241** (orchestrator-direct paperwork). Explicit acknowledgment of:
- Stone 241.6 → 241.10 orphaned commitment closed by Stone 241.14 (25 days late)
- `feedback_defer_by_naming` doctrine memory inscribed
- The def-family death campaign complete (5 stones: 241.12/13/14/15/16)
- 12-entry RETIREMENT_TABLE as historical record
- Scheme → Clojure conversion at the define layer complete

**Arc 237.8b** reopens after Stone 241.17 per `feedback_no_regression_until_arc_done`

**Broader Clojure conversion arcs** queued post-arc-241:
- Arc 172 — comma-to-apostrophe-dispatch
- Arcs 173/174 — clojure macros + features
- Arcs 175/176/177 — enum/struct/defmacro syntax Clojure
- Arc 181 — match syntax Clojure

**The substrate's identity** as Clojure-flavored homoiconic Lisp on Rust crystallizes. The scheme-style legacy is buried in the graveyard (RETIREMENT_TABLE). The def-family is clean: def + defn + defmacro + defstruct + defenum + defclause + defalias. The scaffolding for `define` is GONE — not just rejected at startup but absent from the substrate's recognized form set.
