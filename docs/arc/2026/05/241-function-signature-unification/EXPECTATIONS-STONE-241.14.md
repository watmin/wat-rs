# EXPECTATIONS — Stone 241.14 — `:wat::core::def-restricted` + `:wat::core::defn-restricted` ABSORB INTO METADATA-MAP (Enemy 4 of 4)

Independent scorecard. NO vigilia required (D6 — legacy flat substrate; no new namespaced home). SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-06 PASS 6/6 | `cargo test --release --test probe_arc241_stone14_restricted_absorbed` | 6/0 |
| 2 | Stone 241.13 probe preserved 2/2 | `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut` | 2/0 |
| 3 | Stone 241.12 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone12_defalias` | 5/0 |
| 4 | Stone 242.2 + 242.1 + 241.11 + 241.10 probes preserved | each | counts preserved |
| 5 | arc 198 acceptance test migrates (per S7) | `cargo test --release --test wat_arc198_def_restricted` | passes (forms migrated) |
| 6 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 8 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 925 (looser per STOP-trigger 10) |
| 9 | Pre-INSCRIPTION grep CLEAN (def-restricted) | `grep -rn ":wat::core::def-restricted\b" src/ tests/ wat/` | 0 active matches |
| 10 | Pre-INSCRIPTION grep CLEAN (defn-restricted) | `grep -rn ":wat::core::defn-restricted\b" src/ tests/ wat/` | 0 active matches |
| 11 | RETIREMENT_TABLE has 9 entries | `grep -cE '^    \(":wat' src/remedy/retirement.rs` | 9 entries |
| 12 | SCORE-STONE-241.14.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.md` | file exists |

## Structural verification (12 rows)

| Verification | Command | Expected |
|---|---|---|
| `:wat::core::def-restricted` HARD-CUT arm in check.rs | `grep -n '":wat::core::def-restricted"' src/check.rs` | ≥ 1 match (the HARD-CUT arm) |
| `:wat::core::defn-restricted` HARD-CUT arm in check.rs | `grep -n '":wat::core::defn-restricted"' src/check.rs` | ≥ 1 match (the HARD-CUT arm) |
| 8th + 9th RETIREMENT_TABLE entries verbatim | `grep -A1 'Stone 241.14' src/remedy/retirement.rs` | matches both `(":wat::core::def-restricted", ":wat::core::def")` and `(":wat::core::defn-restricted", ":wat::core::defn")` |
| `defined_value_restrictions` field absent from SymbolTable | `grep -n "defined_value_restrictions" src/runtime.rs` | 0 matches (or only historical comments) |
| `defined_value_restrictions` field absent from CheckEnv | `grep -n "defined_value_restrictions" src/check.rs` | 0 matches (or only historical comments) |
| `set_defined_value_restriction` / `get_defined_value_restriction` methods absent | `grep -rn "set_defined_value_restriction\|get_defined_value_restriction" src/` | 0 matches |
| `binding_metadata`-driven walker | `grep -n "walk_for_restricted_call\|binding_metadata" src/check.rs` | walker function present; reads binding_metadata |
| RestrictionEntry inventory channel STAYS | `grep -n "RestrictionEntry" src/restriction_entry.rs src/freeze.rs` | struct + iter intact |
| RestrictionEntry populates binding_metadata not defined_value_restrictions | `grep -n "binding_metadata" src/freeze.rs` | ≥ 1 site in RestrictionEntry iteration |
| `wat/core.wat` defn-restricted macro DELETED | `grep -n "defn-restricted" wat/core.wat` | 0 matches in active code (only historical comments) |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| No "stays as sugar" framings in source | `grep -rn "stays as sugar\|keep as sugar\|preserved as sugar" src/ \| grep -i "restricted"` | 0 such framings |

## Prediction: 90–180 min Mode A

Stone 241.14 decomposition:
- Walker migration to binding_metadata reads (incl. prefix-list extraction helper) — **~20-30 min**
- defined_value_restrictions storage DELETION cascade — **~30-45 min** (similar to Stone 241.13's DispatchRegistry pattern; substrate-as-teacher)
- RestrictionEntry inventory channel migration — **~10-15 min**
- HARD-CUT arms (2) + RETIREMENT_TABLE entries (2) — **~10 min**
- `wat/core.wat` macro deletion + comment update — **~5 min**
- Test migration (wat_arc198_def_restricted 5 sites + arc170_stone_b 1 site) — **~20-30 min**
- Doc migration (USER-GUIDE + CONVENTIONS) — **~10-15 min**
- Reflection emitter audit — **~5 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 90-180 min. Under-band possible if walker migration is simpler than estimated (binding_metadata storage already populates via Stone 241.6 mechanism).

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `fc5906ef`)

1. Stone 241.14 probe at HEAD: **1/6 PASS** (C01 preservation passes; 5/6 disconfirm)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. Probe 241.13: 2/2 · Probe 241.12: 5/5 · Probe 242.2: 6/6 · Probe 242.1: 4/4 · Probe 241.11: 5/5 · Probe 241.10: 8/8 · prior probes preserved
4. Clippy: **905** (post-Stone-241.13; gate raised to ≤ 925 for substrate-refactor line-shift)
5. RETIREMENT_TABLE: **7 entries** (will grow to 9 — Stone 241.14 S5)
6. `:wat::core::def-restricted` active uses: 5 test sites in wat_arc198_def_restricted.rs + RestrictionEntry inventory channel (substrate-internal) + parser at runtime.rs + check arm at check.rs + walker
7. `:wat::core::defn-restricted` active uses: 2 test sites in wat_arc198_def_restricted.rs + macro definition in wat/core.wat:202-209 + doc references in USER-GUIDE + CONVENTIONS
8. `defined_value_restrictions` parallel storage active across 4 substrate files (runtime.rs + check.rs + freeze.rs + restriction_entry.rs)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Walker prefix-list extraction from metadata-map non-trivial (WatAST::Vector unpacking) | per Stone 241.10 precedent | write small helper adjacent to walker; document |
| **T2** | RestrictionEntry migration breaks arc 170 Stone B Thread/join-result restrictions | post-strike acceptance test | verify Thread/join-result + Process/join-result restrictions still fire from non-allowed callers |
| **T3** | defined_value_restrictions DELETION cascades ~30 sites | compile errors | substrate-as-teacher; mechanical |
| **T4** | Reflection emitter for def-restricted/defn-restricted (analogous to Stone 241.12 trap-door) | grep audit | migrate per S9 |
| **T5** | wat_arc198_def_restricted.rs tests may exercise empty whitelist edge case (per T7 in DESIGN) | per-test review | preserve `:restricted-to []` = "no caller allowed" semantic |
| **T6** | wat/core.wat macro deletion sequencing — macro deletion must happen BEFORE HARD-CUT arm fires, otherwise existing macro callers parse-error pre-rejection | sonnet's cadence ordering | follow BRIEF cadence: walker migration first, storage delete next, THEN HARD-CUT arms, THEN macro delete (cadence step 7) |
| **T7** | "stays as sugar" temptation for defn-restricted | self-audit | STOP per `feedback_hard_cut_admits_no_bypasses`; user direction "def and defn are the only ways" — both retire |
| **T8** | Auto-fixer temptation (cascade scale moderate) | post-strike `ls crates/` | acceptable if EPHEMERAL — built, used, DELETED |
| **T9** | Sonnet drafts INTERSTITIAL | post-strike `git diff INTERSTITIAL-REALIZATIONS.md` | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T10** | SCORE doc not written | post-strike `ls SCORE-STONE-241.14.md` | DISCIPLINE GAP per `feedback_score_present_check_before_closure` |
| **T11** | Stone 241.15 scope creep | post-strike grep for is_mutation_head/parse_define_form touches | STOP per D9; revert |
| **T12** | feedback_defer_by_naming lesson not honored — Stone 241.6 orphan acknowledgment may be missing from SCORE | post-strike SCORE review | SCORE explicitly references the orphaned commitment; doctrine-memory P2 follow-up captures the lesson |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally (orchestrator re-runs each command independently)
- 12/12 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.md`

### Phase B — NOT cast (no vigilia per D6)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/check.rs` (HARD-CUT arms + walker migration + storage deletion), `src/runtime.rs` (storage deletion + populate-path deletions), `src/freeze.rs` (RestrictionEntry migration), `src/restriction_entry.rs` (likely doc comment update), `src/remedy/retirement.rs` (8th + 9th entries), `src/closure_extract.rs` (if reflection emitters touched), `wat/core.wat` (macro deletion + comment update), `tests/wat_arc198_def_restricted.rs` (5 site migration), `tests/wat_arc170_stone_b_walker_collapse.rs` (if migration needed), `docs/USER-GUIDE.md`, `docs/CONVENTIONS.md`, SCORE doc
- INTERSTITIAL NOT in commit (D7; orchestrator authors after Stone 241.16 INSCRIPTION closes arc 241)
- Push to origin
- Stone 241.15 (Enemy 3 — define eval-time residue) opens next; DESIGN drafted after 241.14 ships

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.11 | define HARD CUT + 271-site cascade + auto-fixer | +7957/-9158 | 120-240 min | ~98 min |
| 241.12 | defalias mint native + 13-caller cascade + S6 consistency pass + 2 trap-doors | +622/-229 | 60-150 min | ~130 min + context boundary |
| 241.13 | define-dispatch HARD CUT + src/dispatch.rs DELETED (445 lines) + DispatchRegistry plumbing deletion + 6 test files | +340/-1203 | 90-180 min | **~25 min** (substantially under-band) |
| **241.14 (this)** | **def-restricted + defn-restricted ABSORPTION; defined_value_restrictions storage DELETION; RestrictionEntry inventory migration; walker rewrite; 2 HARD-CUT arms + 2 RETIREMENT_TABLE entries; test migration + doc cascade** | **TBD (probably -200 to +100 net)** | **90-180 min** | **TBD** |

## What this unblocks

**Stone 241.15** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT)

**Stone 241.16** — INSCRIPTION closes arc 241. Explicitly acknowledges Stone 241.6 orphaned commitment (D10 + line 182 → Stone 241.10 → orphaned → Stone 241.14 closes it 25 days late). The `feedback_defer_by_naming` lesson lands.

**Arc 237.8b** — reopens after Stone 241.16 per `feedback_no_regression_until_arc_done`

**The def\*-prefix family** — def + defn are the ONLY definers post-stone. Restrictions are binding-level metadata (`:restricted-to` key). One canonical path per task per `feedback_wat_llm_first_design`.

**Future binding-level metadata** — `:doc`, `:deprecated`, `:since`, `:see-also`, etc. — all consume the same metadata-map mechanism. Stone 241.14 proves the pattern is foundational across ENFORCEMENT (not just reflection).
