# EXPECTATIONS — Stone 241.1.fix — vigilia-convergence amends on `src/argspec/*`

Independent scorecard for orchestrator-side verification after sonnet returns. Each row is a fact to confirm via an explicit command; orchestrator re-runs locally and writes the verbatim result into `SCORE-STONE-241.1.fix.md`.

## Phase A — Scorecard (14 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Probe contracts 1–10 still PASS post-rename | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_0` | 10 passed; 0 failed (matches `contract_0*` prefix) |
| 2 | Probe contract 11 PASS (MalformedTypeKeyword) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_11_malformed_type_keyword` | 1 passed; 0 failed |
| 3 | Probe contract 12 PASS (RetTypeNotKeyword) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_12_ret_type_not_keyword` | 1 passed; 0 failed |
| 4 | Probe contract 13 PASS (IncompleteSignature) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_13_incomplete_signature` | 1 passed; 0 failed |
| 5 | Probe whole-suite PASS 13/13 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 13 passed; 0 failed |
| 6 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher; never < 834) |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 8 | Clippy delta = 0 | `cargo clippy --release 2>&1 \| grep -c "^warning"` | ≤ pre-stone count (orchestrator captures pre-stone count before spawn) |
| 9 | Files touched match discipline | `git diff --name-only HEAD` (uncommitted at sonnet's return) | EXACTLY: `src/argspec/error.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.fix.md` |
| 10 | `src/argspec/mod.rs` UNCHANGED | `git diff src/argspec/mod.rs` | empty diff |
| 11 | `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |
| 12 | `classify()` method present with locked reasons | `grep -n "fn classify" src/argspec/error.rs` | one match; method exists |
| 13 | `parse_keyword_type` helper present (private) | `grep -n "fn parse_keyword_type" src/argspec/parse.rs` | one match; helper exists; NOT prefixed with `pub` |
| 14 | No prior arc 237 probe regresses | `cargo test --release --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | All-suite PASS counts preserved |

## Independent prediction (runtime band)

**Target band: 20–30 min Mode A.**
**Upper bound: 40 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Probe < 13/13 at sonnet return
- Lib baseline < 834
- Clippy warnings increased above pre-stone count
- Files touched outside the discipline (any STOP-5 hit)
- `src/argspec/mod.rs` or `src/lib.rs` touched
- Reason-string drift persists (the three From impls still produce different strings for the same variant)
- Any prior arc 237 probe regression

**Mirror precedent: Stone 241.1** (250-line surface, ~50 min actual). Stone 241.1.fix is a smaller amend pass against the same home (~150 net line delta — substantial code SAVED by `classify()` collapse); the prediction adjusts downward accordingly.

## Trap-door risks (enumerated; orchestrator watches)

| # | Risk | Detection | Resolution if hit |
|---|---|---|---|
| **T1** | Sonnet implements `classify()` but the From impls still duplicate reason strings (didn't replace the match arms) | Inspect `error.rs` post-return for residual reason-string match arms in From impls | Hard re-brief. The classify extraction is load-bearing; the From impls MUST collapse to 4-line wrappers. |
| **T2** | Sonnet leaves the `arg-vector` / `field/arg` prefixes in `classify()`'s reason strings instead of going domain-neutral | Grep `classify()` body for "arg-vector" or "field/arg" | Re-brief — D1 locks domain-neutral language; the head field carries form context |
| **T3** | Sonnet adds `pub` to `parse_keyword_type` or `classify()` | Grep for `pub fn classify` or `pub fn parse_keyword_type` | Re-brief — both are PRIVATE; no public API extension |
| **T4** | Sonnet keeps the opaque `impl Deref<Target=Span>` return in the probe (only renames; doesn't replace) | Grep `impl std::ops::Deref` in probe | Re-brief — A4 mandates owned `(Vec<WatAST>, Span)` |
| **T5** | Sonnet's contract 11 uses a fixture or skips on STOP-10 instead of finding a real malformed-keyword shape | Inspect contract 11 source string + match arm | If STOP-10 hit honestly (no shape exists): orchestrator amends or de-scopes contract 11 in a follow-up. If sonnet faked it: hard re-brief. |
| **T6** | Sonnet keeps the tautological `is_bare_symbol(&args_vec[idx], "->")` guard at the ret-arrow check | Grep `parse.rs` for `if !is_bare_symbol` after the loop | Re-brief — C1 mandates removal |
| **T7** | Sonnet adds new files (e.g., `src/argspec/helpers.rs`) | `ls src/argspec/` post-return | STOP-6 hit. Hard re-brief. |
| **T8** | Sonnet runs a wrapper script (e.g., `./scripts/...`) and reports tool denial | `grep -i "denied\|unavailable" sonnet output` | False claim — bash works for sonnet. Use FM 7 verification protocol if recurring. |
| **T9** | Sonnet adds `.clone()` calls inside the From impls (defeats the classify-consumes-self shape) | Grep `from()` impl bodies for `.clone()` | Re-brief — `classify(self)` moves; no clones needed |
| **T10** | Sonnet rune-defers C1 (tautology) or C2 (saturating_sub) instead of cleaning | Grep for `rune:` markers at parse.rs:99 / 158-163 | Re-brief — the only legitimate runes are A3 (`unreachable!` arm + `rest_param` field) |

## Pre-spawn baseline checks (orchestrator runs BEFORE spawning)

1. **Lib baseline at HEAD = 834 PASS / 0 FAIL.** Verified this turn (this is the floor for row 6).
2. **Probe 10/10 PASS at HEAD.** Verified by the SCORE-STONE-241.1.md scorecard (rows 1–11).
3. **Workspace test-build clean at HEAD.** Verified by SCORE-STONE-241.1.md row 13.
4. **Clippy baseline at HEAD.** Capture exact warning count immediately before spawn; row 8 compares against it.

## What completion looks like (TWO phases — Phase A green is the floor; Phase B convergence is the bar)

### Phase A — SCORE scorecard verification (sonnet's behavioral + structural correctness)

After sonnet returns Mode A:
- 14/14 rows verify locally (orchestrator's independent re-run)
- `SCORE-STONE-241.1.fix.md` written with verbatim row results + honest deltas
- **DO NOT commit yet.** Phase A is the L0 floor — substrate works AND structure converged. The bar is Phase B.

### Phase B — Vigilia re-cast on the namespaced home (per `feedback_namespaced_home_vigilia_gate`)

Once Phase A green, orchestrator casts **vigilia** on the amended `src/argspec/*` + the amended probe. The applicable defensive subset (8 spells in parallel):

| Spell | What this re-cast looks for |
|---|---|
| intueri | Renames hold; `classify` + `parse_keyword_type` speak; new probe contract names self-explain |
| solvere | Reason-string drift ELIMINATED; one source of truth in `classify()`; helper unifies keyword-parse |
| purgare | Runes correctly applied (only A3 sites); no dead code introduced; no orphan helpers |
| struere | `classify()` is one function doing one thing; `parse_keyword_type` is atomic with right scope |
| sequi | `classify(self)` consumes; From impls move; no rogue `.clone()` calls |
| temperare | Tautology gone; saturating_sub form; no redundant work |
| complectens | New contracts compose from `parse_triples`; same shape as 1–10; one helper per test layer |
| vocare | Probe still tests parser as caller invokes it; new contracts exercise real error paths |

**Bar:** L1 + L2 findings = 0. L3 taste noted, not counted. L2 mumbles MAY be accepted via `rune:<spell>(<category>) — <reason>` only when the rune's REASON is load-bearing.

If vigilia finds anything: orchestrator addresses OR directs sonnet to amend; re-cast; iterate until L1+L2=0.

### Phase C — Commit + push (only after Phase A + Phase B both green)

- SCORE doc amended with a **Vigilia Convergence** section listing each spell's verdict + any runes accepted
- Atomic commit covers: `src/argspec/error.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `SCORE-STONE-241.1.fix.md`
- Push to origin
- **Phase 1 foundation IMPECCABLE.** Stone 241.2 (migrate A1/A2/A3 fn parsers) can begin against a home that won't generate maintenance debt.

User direction governing this two-phase structure: *"we raise the bar fucking high for namespaced wat-rs files — we ensure {src,tests}/argspec/ are shockingly good, remarkably well written — the spells ensure this — we do not move from those until we are exceptional."*

## Calibration history reference

| Stone | Class | Surface delta | Actual runtime | Calibration accuracy |
|---|---|---|---|---|
| 236.0 | Mint type + constructors + tests | +150 net | ~25 min | within 25–45 min target |
| 241.1 | Mint parser + types + tests | +519 net | ~50 min | within 30–50 min target |
| 241.1.fix (this) | Amend / extract / cleanup | **-80 to -100 net (saves code)** | TBD | predict: 20–30 min |

Per `feedback_stone_briefs_cite_prior_score`: the precedent informs the band; the net-negative line count (significant savings from `classify()` collapse) justifies the narrower lower band. If 241.1.fix ships substantially over 30 min, the calibration model needs revision before Stone 241.2's BRIEF.
