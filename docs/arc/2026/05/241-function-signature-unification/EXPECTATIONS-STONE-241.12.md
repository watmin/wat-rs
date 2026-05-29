# EXPECTATIONS — Stone 241.12 — `:wat::core::defalias` mint + alias-cascade completion

Independent scorecard. NO vigilia required (D6 — legacy flat substrate; no new namespaced home). SCORE-green commit. Upper bound 150 min.

## Phase A — Scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-03 PASS (defalias works; additive; alias resolves) | `cargo test ... probe_arc241_stone12_defalias` | 3/0 |
| 2 | Probe whole-suite 3/3 | `cargo test --release --test probe_arc241_stone12_defalias` | 3/0 |
| 3 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 4 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 5 | Stone 241.1-241.9 probes + arc 237/238 probes preserved | each | counts preserved |
| 6 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 8 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |
| 9 | Pre-INSCRIPTION grep CLEAN | per S6 in BRIEF; 0 non-acceptable matches | 0 violations |
| 10 | Auto-fixer crate (if minted) DELETED | `ls crates/fix-*/` | "No such file or directory" OR not minted at all |

## Structural verification (7 rows)

| Verification | Command | Expected |
|---|---|---|
| `:wat::core::defalias` recognized in dispatch | `grep -n ":wat::core::defalias" src/types.rs src/check.rs` | ≥ 1 match (the new dispatch entry) |
| `parse_defalias` (or equivalent) function present | `grep -n "fn parse_defalias\|defalias.*parse" src/types.rs` | ≥ 1 match |
| RETIREMENT_TABLE has 5 entries | `grep -c '"' src/remedy/retirement.rs` (count of RETIREMENT_TABLE entries) | 5 entries (struct/struct-restricted/enum/define + NEW runtime define-alias → defalias) |
| `:wat::runtime::define-alias` HARD-CUT-rejected at check.rs (per S2.5) | `grep -n '":wat::runtime::define-alias"' src/check.rs` | ≥ 1 match (the HARD-CUT arm) |
| 26 `:wat::runtime::define-alias` callers migrated to defalias | `grep -rn ":wat::runtime::define-alias\b" src/ wat/ tests/` (excluding retirement.rs + check.rs HARD-CUT arm + historical comments) | 0 active uses |
| No `:wat::core::define` "privileged path" framing | `grep -n "privileged\|bypass\|intentional.*define" src/` (excluding comments documenting retirement) | 0 such framings |
| Pre-INSCRIPTION grep returns only acceptable categories | per S6 protocol | 0 non-acceptable |
| Auto-fixer ephemeral discipline (if applicable) | `ls crates/ \| grep -i "fix-defalias\|fix-define"` | empty |

## Prediction: 60–120 min Mode A

Stone 241.12 is structurally LIGHTER than 241.10 (no schema upgrade) and 241.11 (cascade bounded by substrate alias uses, not 271 user-facing sites).

Per `docs/SUBSTRATE-AS-TEACHER.md`: cascade is the migration brief. Initial fail-count after S1 substrate change: bounded by substrate alias-use count + reflection emitter sites (estimated 10-30 sites). Per-site work: ~30-90 seconds mechanical.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Substrate alias-use cascade larger than estimated (>50 sites) | grep | extend within band; STOP-3 at 150 min if extreme |
| **T2** | Reflection emitter migration requires conditional logic (alias vs function shape) | sonnet surfaces honest delta | per-emitter judgment; document in SCORE |
| **T3** | `:wat::runtime::define-alias` runtime mechanism doesn't cleanly compose with new parser | trap-door | BUILD the missing piece forward per `feedback_trap_door_build_the_dependency`; document |
| **T4** | Pre-INSCRIPTION grep surfaces uses outside Pattern A/B/C categories | sonnet's audit | per-site judgment; if a new category surfaces, document with affirmative-out-of-scope justification |
| **T5** | Auto-fixer (if used) generates wrong shape | precedent: Stone 241.11 T-argspec | use cautiously; manual residuals expected |
| **T6** | Probe C03 requires defalias to actually RESOLVE the alias name (not just register it) | runtime semantics | sonnet may need to extend the runtime resolver if defalias-registered names aren't resolved by existing mechanism |
| **T7** | `:wat::core::define` use is genuinely ambiguous (alias vs function — body is opaque) | per-site review | choose conservatively; document; the structured retirement remedy from Stone 241.10 will guide users either way |
| **T8** | "Privileged path" framing tempts sonnet again | self-audit | STOP-5 (D7 + `feedback_hard_cut_admits_no_bypasses`); the use migrates |

## Pre-spawn baseline checks

1. Stone 241.12 probe at HEAD = 2/3 PASS (C01/C02 trivially via substrate no-op; C03 disconfirms cleanly with UnresolvedReference :app::salutation).
2. Lib at HEAD = 890 PASS / 0 FAIL.
3. All Stone 241.x probes at current counts.
4. Clippy 902 ≤ 902 gate.
5. RETIREMENT_TABLE has 4 entries (will stay 4 — D5).

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 10/10 scorecard verifies locally
- 7/7 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` (NOT at repo root)

### Phase B — NOT cast (no vigilia per D6)

### Phase C — Commit + push

- Atomic commit covers: `src/types.rs` (defalias dispatch + parse), `src/check.rs` (if needed), `src/runtime.rs` (defalias compile/expand to runtime mechanism; reflection emitter migrations), `src/freeze.rs` (if needed), substrate alias-cascade target files, SCORE doc
- src/remedy/* should NOT be modified (D5)
- Push to origin
- Stone 241.13 (INSCRIPTION) opens next; arc 237.8b reopens after

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.9 | defenum HARD CUT + 33-site cascade + R-gap trap-door | 60-120 min | ~50 min |
| 241.10 | src/remedy/ mint + schema HARD CUT + 160-site cascade + 6-round vigilia | 120-180 min ship + 6 rounds | ship within band |
| 241.11 | define HARD CUT + ~271-site cascade + auto-fixer + 2 trap-door fixes | 120-240 min | ~98 min |
| 241.11.fix r1 | Test migrations + 1 doc update | 60-120 min | ~17 min |
| 241.11.fix r2 | Substrate alias migration (KILLED due to wrong framing) | — | killed |
| **241.12 (this)** | **defalias mint + bounded substrate alias cascade** | **60-120 min** | **TBD** |

Stone 241.12 is the SMALLEST substantive stone in arc 241's tail. The cascade is bounded; the mint is simple; the runtime mechanism already exists. Per-stone calibration favors UNDER-band finish.

## What this unblocks

**Stone 241.13** — INSCRIPTION closes arc 241. Pre-INSCRIPTION grep enforced (per FM 11 + Stone S11 of recovery doc). With defalias minted + alias cascade complete, the grep gate passes.

**Arc 237.8b** — reopens after Stone 241.13 per `feedback_no_regression_until_arc_done`.

**The def*-prefix family completes** — def / defn / defclause / defmacro / defstruct / defenum / defalias all shipping; defrecord queued arc 227; deftypealias queued arc 109. The pattern is foundational.
