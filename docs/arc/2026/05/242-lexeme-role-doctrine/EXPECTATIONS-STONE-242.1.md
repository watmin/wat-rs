# EXPECTATIONS — Stone 242.1 — bare `nil` + Char HARD CUT + doctrine inscription

Independent scorecard. NO vigilia required (D6 — legacy flat substrate; no new namespaced home). SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-04 PASS | `cargo test ... probe_arc242_stone1_lexeme_role` | 4/0 |
| 2 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 3 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 4 | Stone 241.1-241.9 probes + arc 237/238 probes preserved | each | counts preserved |
| 5 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 6 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 7 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |
| 8 | RETIREMENT_TABLE has 5 entries | `grep -c "(\":wat::core" src/remedy/retirement.rs` | ≥ 5 matches |
| 9 | Doctrine memory inscribed | `ls ~/.claude/projects/-home-watmin-work-holon/memory/project_lexeme_role_doctrine.md` | file exists |
| 10 | MEMORY.md index updated | `grep -n "lexeme-role-doctrine" ~/.claude/projects/-home-watmin-work-holon/memory/MEMORY.md` | ≥ 1 match |

## Structural verification (7 rows)

| Verification | Command | Expected |
|---|---|---|
| `:wat::core::Char` HARD-CUT arm at check.rs | `grep -n '":wat::core::Char"' src/check.rs` | ≥ 1 match (the rejection arm) |
| `:wat::core::Char` entry in RETIREMENT_TABLE | `grep -n '":wat::core::Char"' src/remedy/retirement.rs` | 1 match (the entry) |
| `:wat::core::char` (lowercase) live as type | `grep -n ":wat::core::char\b" src/` (excluding HARD CUT arm) | ≥ 1 active reference |
| `:wat::core::nil` TYPE-position uses preserved (sample check) | `grep -n "-> :wat::core::nil\|<- :wat::core::nil" src/ tests/ wat/ \| wc -l` | substantial count (type uses stay) |
| Active `:wat::core::Char` uses post-stone | `grep -rn ":wat::core::Char\b" src/ tests/ wat/` (excluding HARD CUT arm + retirement entry + probe + historical comments) | 0 active uses |
| Auto-fixer crate DELETED | `ls crates/fix-*/ 2>&1` | "No such file or directory" |
| No "privileged path" framings | `grep -nE "privileged.*Char\|Char.*intentional" src/ docs/` | 0 such framings |

## Prediction: 60–150 min Mode A

Stone 242.1's cascade is bounded:
- Bare nil lexer work: small (likely 0-50 lines if needed; possibly nothing if already operational)
- `:wat::core::nil` cascade: ~50-150 sites (value-position subset of 255 total)
- `:wat::core::Char` cascade: ~30-100 sites
- RETIREMENT_TABLE append: 1 line
- check.rs HARD-CUT arm: ~10 lines
- Doctrine memory inscription: 1 new file + MEMORY.md update

Per `docs/SUBSTRATE-AS-TEACHER.md`: cascade is the migration brief. Initial fail-count after S3 substrate change: ~30-100 (Char-using sites). Per-site work: ~30-90 seconds mechanical.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Bare nil lexer already operational (S1 unnecessary) | C01 passes at HEAD | document finding in SCORE; skip S1 |
| **T2** | `:wat::core::nil` type vs value ambiguous in some context | per-site judgment | PRESERVE (lean conservative); document in SCORE |
| **T3** | `:wat::core::char` already exists as a name (collision) | C04 passes at HEAD | investigate; if already exists for same semantic, the rename is structural cleanup not new mint |
| **T4** | `:wat::core::Char` cascade includes WAT source files | grep | auto-fixer or per-file migration |
| **T5** | Auto-fixer ephemeral discipline (build → use → DELETE) | git status post-strike | STOP if survives commit |
| **T6** | Reflection emitter migration requires conditional logic | sonnet judges per-emitter | document strategy in SCORE |
| **T7** | Doctrine inscription requires careful prose | orchestrator-direct content | sonnet drafts; orchestrator finalizes; INTERSTITIAL gets prose during orchestrator commit |
| **T8** | "Privileged path" framing tempts sonnet on Char | self-audit | STOP per `feedback_hard_cut_admits_no_bypasses`; the use migrates |

## Pre-spawn baseline checks

1. Stone 242.1 probe at HEAD = 3/4 PASS (C03 disconfirms cleanly; others trivially pass).
2. Lib at HEAD = 890 PASS / 0 FAIL.
3. All Stone 241.x probes + arc 237/238 probes at current counts.
4. Clippy ≤ 902.
5. RETIREMENT_TABLE has 4 entries (will grow to 5).
6. `:wat::runtime::define-alias` still has 26 callers (UNCHANGED — that's Stone 241.12 territory, which is paused; preserve).

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 10/10 scorecard verifies locally
- 7/7 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/242-lexeme-role-doctrine/SCORE-STONE-242.1.md`
- Memory file `project_lexeme_role_doctrine.md` written + MEMORY.md updated
- INTERSTITIAL realization draft prepared (orchestrator finalizes during commit)

### Phase B — NOT cast (no vigilia per D6)

### Phase C — Commit + push

- Atomic commit covers: `src/parser.rs` / `src/edn_shim.rs` (if lexer work), `src/types.rs` (defalias was Stone 241.12; NOT this stone — preserve current state), `src/check.rs` (HARD-CUT arm), `src/remedy/retirement.rs` (5th entry), `src/runtime.rs` (reflection emitter migrations), cascade target files, SCORE doc, doctrine memory file
- Push to origin
- Stone 242.2 INSCRIPTION opens next; arc 241 resumes at Stone 241.12 after 242.2 closes

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.10 | src/remedy/ mint + schema HARD CUT + 160-site cascade + 6-round vigilia | 120-180 min ship + 6 rounds | ship within band |
| 241.11 | define HARD CUT + 271-site cascade + auto-fixer + 2 trap-door fixes | 120-240 min | ~98 min |
| 241.11.fix r1 | Test migrations + 1 doc update | 60-120 min | ~17 min |
| **242.1 (this)** | **Bare nil lexer audit + value-position cascade (~50-150) + Char HARD CUT (~30-100) + doctrine inscription** | **60-150 min** | **TBD** |

Stone 242.1 is smaller than 241.10/241.11 because the cascade is bounded by the audit (value vs type position; not every nil use migrates). The doctrine inscription is the substantive lift.

## What this unblocks

**Stone 242.2** — INSCRIPTION closes arc 242. Orchestrator-direct paperwork (no substrate edits).

**Arc 241 RESUMES** — Stone 241.12 (defalias mint) opens fresh after Stone 242.2 closes. The Stone 241.12 STRIKE-READY artifacts at commit `e803e0f9` stay valid (the BRIEF doesn't reference Char or bare nil; defalias work is orthogonal).

**Stone 241.11.fix work** — round 1's migrations were lost during the 241.12 WIP discard. Sonnet for Stone 241.12 can re-do those migrations along with the defalias work (the BRIEF should be augmented to note this).

**Future case-audits** (Uuid, Time family — queued in arc 109 territory) consume Doctrine 2 (scalar lowercase, container PascalCase) as the rule.

**Future EDN-fidelity work** consumes Doctrine 1 (bare = value; keyword = type) as the rule.
