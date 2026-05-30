# EXPECTATIONS — Stone 241.13 — `:wat::core::define-dispatch` HARD CUT + DispatchRegistry scaffolding deletion (Enemy 2 of 3)

Independent scorecard. NO vigilia required (D5 — legacy flat substrate; no new namespaced home). SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-02 PASS 2/2 | `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut` | 2/0 |
| 2 | Stone 241.12 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone12_defalias` | 5/0 |
| 3 | Stone 242.2 probe preserved 6/6 | `cargo test --release --test probe_arc242_stone2_value_position_doctrine` | 6/0 |
| 4 | Stone 242.1 probe preserved 4/4 | `cargo test --release --test probe_arc242_stone1_lexeme_role` | 4/0 |
| 5 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 6 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 7 | Stone 241.1-241.9 + arc 237/238 probes preserved (except dispatch-dependent: probe_arc237_7a/7b may be deleted/repurposed per S5) | each | counts preserved or expected deltas documented |
| 8 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL (expected delta from test file deletions; track) |
| 9 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 10 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 920 (looser per STOP-trigger 10) |
| 11 | Pre-INSCRIPTION grep CLEAN | `grep -rn ":wat::core::define-dispatch\b" src/ tests/ wat/` per S9 categories | 0 active matches |
| 12 | SCORE-STONE-241.13.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md` | file exists |

## Structural verification (10 rows)

| Verification | Command | Expected |
|---|---|---|
| `:wat::core::define-dispatch` HARD-CUT arm in check.rs | `grep -n '":wat::core::define-dispatch"' src/check.rs` | ≥ 1 match (the HARD-CUT arm) |
| 7th RETIREMENT_TABLE entry verbatim | `grep -A2 'Stone 241.13' src/remedy/retirement.rs` | matches `(":wat::core::define-dispatch", ":wat::core::defclause")` |
| `src/dispatch.rs` DELETED | `ls src/dispatch.rs 2>&1` | "No such file" |
| `DispatchRegistry` symbol absent from substrate | `grep -rn "DispatchRegistry" src/` | 0 matches |
| `dispatch_registry` field absent from CheckEnv | `grep -n "dispatch_registry" src/check.rs` | 0 matches (or only historical comments) |
| `dispatch_registry` field absent from SymbolTable | `grep -n "dispatch_registry" src/runtime.rs` | 0 matches (or only historical comments) |
| `infer_dispatch_call` symbol absent | `grep -rn "infer_dispatch_call" src/` | 0 matches |
| `:wat::core::define-dispatch` absent from special_forms registry | `grep -n "define-dispatch" src/special_forms.rs` | 0 matches |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| No "infrastructure stays empty" framing in source | `grep -rn "stays empty\|registry empty" src/` | 0 such framings |

## Prediction: 90–180 min Mode A

Stone 241.13 decomposition:
- HARD CUT arm + 7th RETIREMENT_TABLE entry — **~10 min**
- `src/dispatch.rs` DELETE (`git rm`) — **~1 min**
- DispatchRegistry plumbing deletion cascade — **~30-45 min** (substrate-as-teacher: each compile error names the next site)
- Test migration/deletion (6 files; per-file judgment) — **~30-60 min** (the dominant variable; per-test analysis time)
- Reflection emitter audit (likely zero work) — **~5 min**
- `wat/core.wat:8` comment update — **~2 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 90-180 min. Under-band possible if test migration is simpler than estimated.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `63adb4ba`)

1. Stone 241.13 probe at HEAD: **0/2 PASS** (both contracts disconfirm cleanly)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. Probe 241.12: 5/5 · Probe 242.2: 6/6 · Probe 242.1: 4/4 · Probe 241.11: 5/5 · Probe 241.10: 8/8 · prior probes preserved
4. Clippy: **908** (post-Stone-241.12; +6 over 902 baseline; accepted per user direction)
5. RETIREMENT_TABLE: **6 entries** (will grow to 7 — Stone 241.13 D2)
6. `src/dispatch.rs`: **445 lines** (will be DELETED)
7. Active `:wat::core::define-dispatch` wat-source uses: **0** (only 1 STALE comment at wat/core.wat:8 referring to the mechanism)
8. Test files referencing define-dispatch: **6** (wat_arc146_dispatch_mechanism, probe_arc237_7a, probe_arc237_7b, wat_arc144_uniform_reflection, probe_declaration_form_lift, probe_def_not_special)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | DispatchRegistry deletion cascades through CheckEnv/SymbolTable (~30-50 sites) | compile errors | per substrate-as-teacher; mechanical |
| **T2** | `infer_dispatch_call` may be referenced by code paths not in initial scope | grep | grep before deletion; cascade to callers |
| **T3** | Test deletion may surface dependencies in other test files (helpers/constants) | build cycle | re-build after each deletion |
| **T4** | `probe_arc237_7a/7b` may be the only regression coverage for evacuated ops | check intrinsic test coverage | if covered elsewhere: delete; if not: REPURPOSE to test intrinsic path |
| **T5** | `wat_arc146_dispatch_mechanism` deletion vs HARD-CUT repurpose decision | sonnet judges | prefer 1-2-contract repurpose to test HARD CUT + remedy; delete rest |
| **T6** | `closure_extract.rs` reflection emitter for define-dispatch (analogous to Stone 241.12 trap-door) | grep `Keyword.*define-dispatch` in src/ | migrate or delete per audit |
| **T7** | Sonnet "infrastructure stays empty" temptation (Stone 241.11.fix round 2 lesson family) | self-audit | STOP per D1 + `feedback_hard_cut_admits_no_bypasses` |
| **T8** | Auto-fixer temptation (cascade scale) | post-strike `ls crates/` | acceptable if EPHEMERAL — built, used, DELETED |
| **T9** | Sonnet drafts INTERSTITIAL | post-strike `git diff INTERSTITIAL-REALIZATIONS.md` | revert per `feedback_sonnet_never_drafts_interstitial` |
| **T10** | SCORE doc not written | post-strike `ls SCORE-STONE-241.13.md` | DISCIPLINE GAP — orchestrator catches; sonnet's cadence must include S10 |
| **T11** | Stone 241.14 scope creep — sonnet touches is_mutation_head/parse_define_form | post-strike grep | STOP per D8; revert any such touches |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally (orchestrator re-runs each command independently)
- 10/10 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md`

### Phase B — NOT cast (no vigilia per D5)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/dispatch.rs` DELETED (`git rm`), `src/check.rs` (HARD-CUT arm + DispatchRegistry deletion), `src/freeze.rs` (plumbing deletion), `src/runtime.rs` (plumbing deletion + form constructor deletion), `src/resolve.rs` (consultation deletion), `src/special_forms.rs` (entry deletion), `src/remedy/retirement.rs` (7th entry), test files (per S5 judgment), `wat/core.wat` (historical comment update), SCORE doc
- INTERSTITIAL NOT in commit (D7; orchestrator authors after Stone 241.15 INSCRIPTION)
- Push to origin
- Stone 241.14 (Enemy 3 — define eval-time residue) opens next

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.11 | define HARD CUT + 271-site cascade + auto-fixer | +7957/-9158 | 120-240 min | ~98 min |
| 242.1 | Char HARD CUT + ~18-site cascade + doctrine memory | mixed | 60-150 min | within band |
| 242.2 | Doctrine 1 SELF-ENFORCING + 166-file cascade | +432/-362 | 60-180 min | within band (SCORE-at-strike gap) |
| 241.12 | defalias mint native + 13-caller cascade + S6 consistency pass + 2 trap-doors | +622/-229 | 60-150 min | ~130 min + context boundary |
| **241.13 (this)** | **define-dispatch HARD CUT + src/dispatch.rs DELETED (~445 lines) + DispatchRegistry plumbing deletion cascade + 6 test files per-file judgment** | **substantial deletion (probably +50/-700 net)** | **90-180 min** | **TBD** |

## What this unblocks

**Stone 241.14** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT)

**Stone 241.15** — INSCRIPTION closes arc 241

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`

**The dispatch entity-kind family** — arc 146's "dispatch by arity + type" mechanism RETIRES; arc 237.2's defclause is the surviving entity kind. The substrate's dispatch story collapses to ONE form per `feedback_wat_llm_first_design`.
