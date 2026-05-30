# EXPECTATIONS — Stone 241.17 — `:wat::core::defmacro` signature migration to canonical (closes arc 177)

Independent scorecard. NO vigilia required. SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-03 PASS 3/3 | `cargo test --release --test probe_arc241_stone17_defmacro_canonical` | 3/0 |
| 2 | Stone 241.16 probe preserved 4/4 | `cargo test --release --test probe_arc241_stone16_define_eval_residue` | 4/0 |
| 3 | Stone 241.15 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone15_zombie_purge` | 6/0 |
| 4 | Stone 241.14 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone14_restricted_absorbed` | 6/0 |
| 5 | Stone 241.10-13 + arc 242 + arc 237/238 probes preserved | each | counts preserved |
| 6 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL (test migrations may shift; document) |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 8 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 940 |
| 9 | Pre-INSCRIPTION grep CLEAN (old shape gone from wat/) | per S9 protocol | 0 old-shape defmacro forms in wat-source |
| 10 | RETIREMENT_TABLE UNCHANGED at 12 entries | `grep -cE '^    \(":wat' src/remedy/retirement.rs` | 12 entries (no new entry — shape-internal rejection) |
| 11 | `parse_defmacro_signature` DELETED | `grep -n "fn parse_defmacro_signature" src/macros.rs` | 0 matches |
| 12 | SCORE-STONE-241.17.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.17.md` | file exists |

## Structural verification (10 rows)

| Verification | Command | Expected |
|---|---|---|
| `parse_defmacro_form` routes through canonical parser | `grep -A30 "fn parse_defmacro_form" src/macros.rs \| grep "parse_argspec_triples"` | ≥ 1 match |
| HARD-CUT-rejection arm for old shape | `grep -n "Stone 241.17\|old defmacro signature shape" src/macros.rs` | ≥ 1 match |
| `parse_defmacro_signature` symbol absent | `grep -rn "parse_defmacro_signature" src/` | 0 matches |
| 29 wat/ migrations landed | `git diff --stat wat/` | 7-15 files modified (29 sites across them) |
| defn macro at wat/core.wat:180 migrated | `grep -A6 "defmacro :wat::core::defn" wat/core.wat` | new shape Vector-triple visible |
| Tests migrations landed | `git diff --stat tests/` | 10+ files modified |
| Doc migrations landed | `git diff --stat docs/USER-GUIDE.md docs/CLOJURE-ROSETTA.md` | both modified (active examples updated) |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| No "compatibility shim" framing | `grep -rn "compatibility\|backward.compat\|keep old form" src/macros.rs` | 0 such framings |
| Reflection emitter audit clean | `grep -n "Keyword.*defmacro" src/closure_extract.rs` | only acceptable patterns (NEW shape emitters or zero matches) |

## Prediction: 90–180 min Mode A

Stone 241.17 scope decomposition:
- `parse_defmacro_form` rewrite + `parse_defmacro_signature` deletion + HARD-CUT arm — **~20-30 min**
- 29 wat/ defmacro migrations (per-file) — **~30-45 min** (varied complexity; defn macro at core.wat:180 is load-bearing — verify after that single migration)
- 36 tests/ references (per-file judgment; some are wat-source strings, some AST construction) — **~30-45 min**
- Doc cascade (USER-GUIDE + CLOJURE-ROSETTA + INTENTIONS) — **~10-15 min**
- Reflection emitter audit — **~5-10 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 90-180 min. Recent stones (241.13/14/15/16/14.fix) all under-band; this one is BIGGEST cascade (65+ sites across wat + tests + docs) so closer to middle-band. The mechanical shape transformation is well-understood; the auto-fixer-with-parser pattern from Stone 241.10 may apply.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `09d7346d`)

1. Stone 241.17 probe at HEAD: **0/3 PASS** (all 3 contracts disconfirm)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. All prior probes preserved
4. Clippy: **897** (post-Stone-241.16; gate raised to ≤ 940)
5. `parse_defmacro_form` at src/macros.rs:320; `parse_defmacro_signature` at line 355 (~80 lines)
6. 29 defmacro callers in wat/ (verified via grep count)
7. 36 tests/ files reference `:wat::core::defmacro` (some in fixtures, some in comments)
8. wat/core.wat:180 = THE defn macro (LOAD-BEARING — all defn callers depend on it expanding correctly)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | wat/core.wat:180 defn macro migration breaks lib + ALL defn callers | post-migration `cargo test --release --lib -p wat` | migrate FIRST; verify lib green before proceeding to other wat/ migrations |
| **T2** | Bulk-sed pattern matching unintended sites (multi-line variations) | per-file diff review | per-file edit safer than blanket bulk-sed |
| **T3** | Auto-fixer-with-parser temptation | post-strike `ls crates/` | acceptable if EPHEMERAL (built, used, DELETED); per Stone 241.10 precedent |
| **T4** | Tests with quoted wat-source strings vs AST-construction code | per-file audit | sonnet distinguishes; migrate per pattern |
| **T5** | Reflection emitters for defmacro (Stone 241.12/13/14/15/16 trap-door class) | grep audit | migrate per S7 |
| **T6** | Sonnet "preserve old shape as sugar" temptation | self-audit | STOP per `feedback_hard_cut_admits_no_bypasses` |
| **T7** | Item-count discriminator (3 = old; 6 = new) edge cases (4 or 5 items = malformed) | sonnet handles via existing MalformedDefmacro path | preserve clean error per-shape |
| **T8** | Metadata-map at items[2] for defmacro — sonnet ships now or defers | per D5 in DESIGN | basic 6-item form first; metadata-map deferred if scope creeps |
| **T9** | Sonnet writes to INTERSTITIAL | post-strike grep | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T10** | SCORE doc not written | post-strike `ls SCORE-STONE-241.17.md` | DISCIPLINE GAP |
| **T11** | Stone 241.18 INSCRIPTION scope creep | post-strike grep | STOP; INSCRIPTION is orchestrator-direct |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally
- 10/10 structural rows verify locally
- SCORE doc at arc dir

### Phase B — NOT cast (no vigilia)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/macros.rs` (parse_defmacro_form rewrite + parse_defmacro_signature deletion + HARD-CUT arm), `src/closure_extract.rs` (if reflection emitters touched), wat/ files (~7-15 modified), tests/ files (~10+ modified), docs/USER-GUIDE.md + docs/CLOJURE-ROSETTA.md + docs/INTENTIONS.md, SCORE doc
- INTERSTITIAL NOT in commit
- Push to origin
- **Stone 241.18 (INSCRIPTION; orchestrator-direct) opens next**

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.12 | defalias mint native + 13-caller cascade + S6 consistency pass | +622/-229 | 60-150 min | ~130 min + context boundary |
| 241.13 | define-dispatch HARD CUT + 445-line file DELETED + 6 test files | +340/-1203 | 90-180 min | ~25 min |
| 241.14 | def-restricted + defn-restricted absorption + storage deletion + walker rewrite | +768/-739 | 90-180 min | ~26 min |
| 241.15 | 3 zombies HARD CUT + dispatch arm deletions + doc cascade | +329/-200 | 60-120 min | ~8.7 min |
| 241.16 | parse_define_form DELETED (~30 sites) + form-predicate arms + 2 trap-doors | +485/-600 | 90-180 min | ~33.8 min |
| 241.14.fix | restriction_entry.rs doc rewrite + types.rs T2 audit | +128/-26 | 10-30 min | ~3.5 min |
| **241.17 (this)** | **parse_defmacro_signature DELETED + parse_defmacro_form rewrite + HARD-CUT arm + 29 wat/ migrations + 36 tests/ migrations + doc cascade** | **TBD (probably +200/-400 net)** | **90-180 min** | **TBD** |

## What this unblocks

**Stone 241.18 — INSCRIPTION closes BOTH arc 241 + arc 177.** Explicit acknowledgment:
- Arc 177 ABSORBED into arc 241 (Stone 241.17 filled the TBD design)
- def-family parser unification GENUINELY COMPLETE — fn/defn/defclause/defmacro all route through `parse_argspec_triples`
- 7-stone campaign (Stones 241.12 + 13 + 14 + 14.fix + 15 + 16 + 17) plus INSCRIPTION at 241.18
- Stone 241.6 → 241.10 orphaned commitment closed by 241.14 (25 days late)
- 12-entry RETIREMENT_TABLE (no new entry from 241.17 — shape-internal rejection)
- Multiple under-band strikes (calibration milestone)
- Scheme → Clojure conversion at def-family layer DONE

**Arc 237.8b** reopens after Stone 241.18 per `feedback_no_regression_until_arc_done`

**Remaining Clojure conversion arcs** queued post-arc-241:
- Arc 172 — comma-to-apostrophe-dispatch
- Arcs 173/174 — clojure macros + features (arc 177 was the sibling; absorbed here)
- Arcs 175/176 — enum/struct syntax Clojure
- Arc 181 — match syntax Clojure

**THE def-family is GENUINELY canonical post-stone:** def + defn + defmacro + defstruct + defenum + defclause + defalias all live native + use canonical parsers + share metadata-map mechanism + share retirement-remedy apparatus. The arc 241 + arc 177 + arc 198 + arc 203 + arc 210 + arc 142 + arc 143 + arc 166 + arc 167 + arc 174 + arc 232's collective vision lands here.
