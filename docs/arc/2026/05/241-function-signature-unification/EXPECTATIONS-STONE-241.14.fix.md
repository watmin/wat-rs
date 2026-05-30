# EXPECTATIONS — Stone 241.14.fix — `src/restriction_entry.rs` doc-comment rewrite

Independent scorecard. Doc-only stone — NO probe (FM 2-bis doesn't apply; structural verification substitutes). SCORE-green commit. Upper bound 30 min.

## Phase A — Scorecard (7 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Lib baseline preserved | `cargo test --release --lib -p wat` | 890 PASS / 0 FAIL (unchanged) |
| 2 | Stone 241.16 probe preserved 4/4 | `cargo test --release --test probe_arc241_stone16_define_eval_residue` | 4/0 |
| 3 | Stone 241.15 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone15_zombie_purge` | 6/0 |
| 4 | Stone 241.14 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone14_restricted_absorbed` | 6/0 |
| 5 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 6 | cargo doc clean | `cargo doc --release --no-deps 2>&1 \| grep -c "^warning:"` | no new warnings |
| 7 | SCORE-STONE-241.14.fix.md authored | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.fix.md` | file exists |

## Structural verification (6 rows)

| Verification | Command | Expected |
|---|---|---|
| Stale "wat-side def-restricted form" phrase GONE from restriction_entry.rs | `grep -n "wat-side .*def-restricted form" src/restriction_entry.rs` | 0 matches |
| Stale "defined_value_restrictions" reference GONE | `grep -n "defined_value_restrictions" src/restriction_entry.rs` | 0 matches |
| Stale "validate_def_restricted_caller_namespace" reference GONE | `grep -n "validate_def_restricted_caller_namespace" src/restriction_entry.rs` | 0 matches |
| `binding_metadata` referenced (current store) | `grep -n "binding_metadata" src/restriction_entry.rs` | ≥ 1 match |
| `walk_for_restricted_call` referenced (current walker) | `grep -n "walk_for_restricted_call" src/restriction_entry.rs` | ≥ 1 match |
| `:restricted-to` referenced (metadata-map key) | `grep -n ":restricted-to" src/restriction_entry.rs` | ≥ 1 match |

## Prediction: 10-30 min Mode A

Stone 241.14.fix scope decomposition:
- Module-level doc comment rewrite (~30 lines) — **~10 min**
- Struct + field doc comment rewrite (~15 lines) — **~5 min**
- T2 audit (other files with similar stale framings) — **~5 min**
- cargo doc verification — **~5 min**
- SCORE doc authoring — **~5 min**

Within-band: 10-30 min. SMALLEST stone in arc 241. No code changes; no cascade; no test migration.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `dbb30979`)

1. Lib at HEAD: **890 PASS / 0 FAIL**
2. All Stone 241.x probes preserved
3. `src/restriction_entry.rs` is 70 lines; ~50 are doc comments (large fraction)
4. Stale phrases counted at HEAD:
   - `defined_value_restrictions` in restriction_entry.rs: 2 sites (lines 30, 32)
   - `validate_def_restricted_caller_namespace` in restriction_entry.rs: 1 site (line 33)
   - `wat-side ... def-restricted form` in restriction_entry.rs: 2 sites (lines 4, 65)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Intra-doc links to deleted symbols (e.g., `[validate_def_restricted_caller_namespace]`) cause cargo doc warnings | `cargo doc --release --no-deps 2>&1` | fix broken intra-doc links |
| **T2** | Similar doc-stale patterns in OTHER files (beyond restriction_entry.rs) | grep audit per BRIEF S3 | if any surface, expand stone scope OR queue Stone 241.14.fix.2 |
| **T3** | Sonnet writes to INTERSTITIAL | post-strike `git diff INTERSTITIAL-REALIZATIONS.md` | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T4** | SCORE doc not written | post-strike `ls SCORE-STONE-241.14.fix.md` | DISCIPLINE GAP |
| **T5** | Sonnet touches Stone 241.17 INSCRIPTION work | post-strike grep | STOP per D6; revert |
| **T6** | Sonnet touches code in restriction_entry.rs (struct field signature, inventory call) | git diff scope check | revert; doc-only scope per D1 |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 7/7 scorecard verifies locally
- 6/6 structural rows verify locally
- SCORE doc at arc dir

### Phase B — NOT cast (no vigilia)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/restriction_entry.rs` (doc rewrite), maybe additional src/ files if T2 surfaces, SCORE doc
- INTERSTITIAL NOT in commit
- Push to origin
- Stone 241.17 (INSCRIPTION; orchestrator-direct) opens next

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.13 | substrate scaffolding deletion | +340/-1203 | 90-180 min | ~25 min |
| 241.14 | substrate refactor + walker migration | +768/-739 | 90-180 min | ~26 min |
| 241.15 | zombie purge | +329/-200 | 60-120 min | ~8.7 min |
| 241.16 | parse_define_form deletion + 2 trap-doors | +485/-600 | 90-180 min | ~33.8 min |
| **241.14.fix (this)** | **doc rewrite of restriction_entry.rs (~50 lines of doc comments)** | **likely +30/-20 net** | **10-30 min** | **TBD** |

The SMALLEST stone of arc 241's tail. Under-band almost certain.

## What this unblocks

**Stone 241.17 — INSCRIPTION closes arc 241** (orchestrator-direct paperwork). The doc-staleness orphan is closed; the def-family death campaign is genuinely complete; the cemetery is truly tidied.

**Doctrine memory `feedback_defer_by_naming`** (pending P2 inscription in Stone 241.17 or as standalone memory) explicitly cites THIS .fix stone as a worked example of "cascade audit must include module-level doc comments, not just code paths." Stone 241.14's cascade missed restriction_entry.rs doc comments because the file's CODE was correctly migrated (transparent populate-target swap); the user-facing doc text describing the OLD substrate slipped past the FM 9 verification + the pre-INSCRIPTION grep gate.

**The pattern lesson for future stones**: when a substrate refactor migrates a populate-target / walker-name / storage-name, the doc comments REFERENCING those names need explicit audit, not just the code paths that USE them.
