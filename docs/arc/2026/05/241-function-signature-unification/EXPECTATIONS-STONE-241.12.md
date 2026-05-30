# EXPECTATIONS — Stone 241.12 — `:wat::core::defalias` mint + `:wat::runtime::define-alias` HARD CUT (Enemy 1 of 3)

Independent scorecard. NO vigilia required (D6 — legacy flat substrate; no new namespaced home). SCORE-green commit. Upper bound 150 min.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-05 PASS 5/5 | `cargo test --release --test probe_arc241_stone12_defalias` | 5/0 |
| 2 | Stone 242.2 probe preserved 6/6 | `cargo test --release --test probe_arc242_stone2_value_position_doctrine` | 6/0 |
| 3 | Stone 242.1 probe preserved 4/4 | `cargo test --release --test probe_arc242_stone1_lexeme_role` | 4/0 |
| 4 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 5 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 6 | Stone 241.1-241.9 + arc 237/238 probes preserved | each | counts preserved |
| 7 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 8 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 9 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |
| 10 | Pre-INSCRIPTION grep CLEAN | `grep -rn ":wat::runtime::define-alias\b" src/ tests/ wat/` (categorize per S8) | 0 active matches |
| 11 | RETIREMENT_TABLE has 6 entries | `grep -c '^    (":wat::core\|^    (":wat::runtime' src/remedy/retirement.rs` | 6 LHS matches |
| 12 | SCORE-STONE-241.12.md authored at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` | file exists |

## Structural verification (8 rows)

| Verification | Command | Expected |
|---|---|---|
| `:wat::core::defalias` recognized in dispatch | `grep -n ":wat::core::defalias\|defalias" src/types.rs src/check.rs` | ≥ 1 active match (the new dispatch entry + native parser) |
| `parse_defalias` (or equivalent native function) present | `grep -n "fn parse_defalias\|defalias.*parse\|register_defalias" src/types.rs src/runtime.rs src/check.rs` | ≥ 1 match |
| `:wat::runtime::define-alias` HARD-CUT arm in check.rs | `grep -n '":wat::runtime::define-alias"' src/check.rs` | ≥ 1 match (the HARD-CUT arm) |
| 6th RETIREMENT_TABLE entry verbatim | `grep -A2 'Stone 241.12' src/remedy/retirement.rs` | matches `(":wat::runtime::define-alias", ":wat::core::defalias")` |
| `wat/runtime.wat:18` macro DELETED | `grep -n "define-alias" wat/runtime.wat` | 0 lines (or only historical comments referencing the retirement) |
| Native implementation, NOT wat-macro intermediate | confirmed by S1 architecture + grep `wat/` for any defalias macro definition | 0 wat-source defalias definitions (only call sites) |
| Auto-fixer crate DELETED if minted | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| No "privileged path" framing in source | `grep -rn "privileged\|intentional bypass\|substrate-internal" src/ \| grep -i "define"` | 0 such framings |

## Prediction: 60–150 min Mode A

Stone 241.12 cascade decomposition:
- Native defalias parser + registrar implementation (~50-100 lines mirroring defstruct/defenum patterns) — **~20-30 min**
- 13-caller mechanical migration — **~15-20 min**
- S5 reflection emitter audit — **~5-10 min**
- S6 consistency pass (~24 test sites + ~10 docs) — **~30-45 min** (the dominant variable)
- HARD CUT arm + RETIREMENT_TABLE append — **~10 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 60-150 min. Under-band possible if consistency-pass count is over-estimated.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `28ab83ef`)

1. Stone 241.12 probe at HEAD: **0/5 PASS** (all 5 contracts disconfirm cleanly)
2. Lib at HEAD: **890 PASS / 0 FAIL**
3. Probe 242.2: 6/6 · Probe 242.1: 4/4 · Probe 241.11: 5/5 · Probe 241.10: 8/8 · prior probes preserved
4. Clippy: **902** at gate ≤ 902
5. RETIREMENT_TABLE: **5 entries** (will grow to 6 — D5)
6. `:wat::runtime::define-alias` active surface uses: **13** (4 wat/core + 2 wat/list + 6 tests + 1 wat/runtime macro impl) per fresh grep
7. `:wat::core::define` in tests/: **~24 sites** (fold-in lost work scope)
8. `:wat::core::define` in docs/: **~10 sites** (fold-in lost work scope)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Built-in aliasing requires registry extension | probe C03 fails post-S1 | per `feedback_trap_door_build_the_dependency` — extend registry; do NOT special-case |
| **T2** | `wat/runtime.wat:18` macro deletion breaks composing macros | grep wat/ for define-alias macro consumers; cascade errors | migrate consumers; document |
| **T3** | Cascade migration touches wat/runtime.wat substrate root | careful editing required | substrate bootstrap; verify startup still loads correctly |
| **T4** | Test fixture uses define-as-alias (rare) | per-site review during S6 | judge → defalias |
| **T5** | Reflection emitter produces define-alias AST | grep src/ for Keyword.*runtime::define-alias | migrate per S5 |
| **T6** | Sonnet self-audit of "privileged path" temptation | post-strike SCORE review | STOP per D7 + `feedback_hard_cut_admits_no_bypasses` — Stone 241.11.fix round 2 was killed for this; do not repeat |
| **T7** | Auto-fixer crate temptation (consistency pass scale) | post-strike `ls crates/` | acceptable if EPHEMERAL — built, used, DELETED before commit |
| **T8** | Sonnet drafts INTERSTITIAL despite explicit prohibition | `git diff docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` post-strike | revert per `feedback_sonnet_never_drafts_interstitial`; orchestrator authors later |
| **T9** | SCORE doc not written at strike end | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` post-strike | DISCIPLINE GAP per `feedback_score_present_check_before_closure` — sonnet must include S9 in cadence; orchestrator's post-strike check catches violation |

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally (orchestrator re-runs each command independently)
- 8/8 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` (NOT at repo root)

### Phase B — NOT cast (no vigilia per D6)

### Phase C — Commit + push (orchestrator)

- Atomic commit covers: `src/types.rs` (defalias dispatch + parse routing), `src/check.rs` (HARD-CUT arm + dispatch), `src/runtime.rs` (defalias native parser + registrar; reflection emitter migrations), `src/freeze.rs` (if needed), `src/remedy/retirement.rs` (6th entry), `src/closure_extract.rs` (if reflection emitters touched), `src/stdlib.rs` (if list reduce/fold registration moves to native), `wat/core.wat` (4 migrations), `wat/list.wat` (2 migrations), `wat/runtime.wat` (macro deletion), cascade test files (~24), cascade doc files (~10), SCORE doc
- INTERSTITIAL NOT in commit (D9; orchestrator authors after Stone 241.15 INSCRIPTION closes arc)
- Push to origin
- Stone 241.13 (Enemy 2 — define-dispatch HARD CUT) opens next; DESIGN drafted after 241.12 ships

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.9 | defenum HARD CUT + 33-site cascade + R-gap trap-door | +809/-576 | 60-120 min | ~50 min |
| 241.10 | src/remedy/ mint + schema HARD CUT + 160-site cascade + 6-round vigilia | mixed | 120-180 min + 6 rounds | ship within band |
| 241.11 | define HARD CUT + 271-site cascade + auto-fixer | +7957/-9158 | 120-240 min | ~98 min |
| 241.11.fix r1 | Test migrations + 1 doc update (LOST during 241.12 WIP discard) | small | 60-120 min | ~17 min (then lost) |
| 241.11.fix r2 | Substrate alias migration (KILLED — wrong "privileged path" framing) | — | — | killed |
| 242.1 | Char HARD CUT + ~18-site cascade + doctrine memory | mixed | 60-150 min | within band |
| 242.2 | Doctrine 1 SELF-ENFORCING + 166-file cascade via FM 16 orchestrator-direct sed | +432/-362 | 60-180 min | within band (no SCORE-at-strike — discipline gap) |
| **241.12 (this — STRIKE-READY-v2)** | **defalias mint native + 13-caller cascade + S6 consistency pass + 6th RETIREMENT_TABLE entry + HARD CUT arm** | **TBD** | **60-150 min** | **TBD** |

## What this unblocks

**Stone 241.13** — Enemy 2 (`:wat::core::define-dispatch` HARD CUT; pure substrate scaffolding deletion; wat-source callers already migrated to ∀T intrinsics per arc 237.7)

**Stone 241.14** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT)

**Stone 241.15** — INSCRIPTION closes arc 241

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`

**The def\*-prefix family completes** — def / defn / defclause / defmacro / defstruct / defenum / defalias all shipping NATIVE
