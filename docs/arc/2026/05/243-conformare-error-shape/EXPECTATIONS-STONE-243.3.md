# EXPECTATIONS — Stone 243.3 — TypeError Pattern A retrofit

Independent scorecard. Two phases: Phase A (sonnet — substrate refactor + cascade); Phase B (orchestrator — conformare re-cast on `src/types.rs` to attest Pattern A conformance). Stone commits ONLY after Phase B attests.

## Phase A scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | FM 2-bis probe COMPILES + PASSES post-strike | `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a` | 3/0 |
| 2 | Lib baseline preserved | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 3 | tests/function preserved | `cargo test --release --test function` | 8/0 |
| 4 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 5 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 945 |
| 6 | TypeError struct shape | `grep -nE "^pub struct TypeError" src/types.rs` | 1 match: `pub struct TypeError {` |
| 7 | TypeErrorKind enum exists | `grep -nE "^pub enum TypeErrorKind" src/types.rs` | 1 match |
| 8 | Old TypeError enum GONE | `grep -nE "^pub enum TypeError\b" src/types.rs` | 0 matches |
| 9 | Conformare rune on CyclicSubtype | `grep -n "rune:conformare" src/types.rs` | ≥ 1 match preceding CyclicSubtype variant |
| 10 | SCORE Phase A authored | `ls docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.3.md` | file exists with Phase A section |

## Structural verification (12 rows)

| Verification | Command | Expected |
|---|---|---|
| TypeError outer struct has span field | `grep -A2 "^pub struct TypeError" src/types.rs` | shows `pub span: Span` |
| TypeError outer struct has kind field | `grep -A3 "^pub struct TypeError" src/types.rs` | shows `pub kind: TypeErrorKind` |
| TypeErrorKind variants do NOT carry span field | `awk '/^pub enum TypeErrorKind/,/^}/' src/types.rs \| grep -c "span:"` | 0 |
| 16 variants in TypeErrorKind | `awk '/^pub enum TypeErrorKind/,/^}/' src/types.rs \| grep -cE "^\s+[A-Z][a-zA-Z]+ \{"` | 16 |
| BadRetType arm collapsed (parse.rs 16-arm match GONE) | `grep -c "TypeError::MalformedTypeExpr { span, .. } => span.clone()" src/function/parse.rs` | 0 |
| BadRetType arm uses err.span | `grep -B1 -A4 "ParseStep::BadRetType" src/function/parse.rs` | shows `e.span` field access |
| WHY comment for old 16-arm match GONE | `grep -c "each TypeError variant carries its own span field" src/function/parse.rs` | 0 |
| From<TypeError> for StartupError preserves span | `grep -A5 "impl From<TypeError> for StartupError" src/freeze.rs` | uses `e.span` |
| Display impl matches on self.kind | `grep -A2 "impl Display for TypeError" src/types.rs \| head -3` (the match shape) | uses `&self.kind` |
| No backward-compat aliases | `grep -n "type TypeError =" src/` | 0 matches |
| No deferral-language additions (exigere preflight) | `grep -rnE "future arc\|outside scope\|would require\|intentionally\|TODO" src/types.rs src/function/parse.rs` (in NEW/CHANGED code) | 0 matches in stone-touched code |
| CONFORMARE.md cites Stone 243.3 + TypeError as first applied example | `grep -n "Stone 243.3\|TypeError" docs/CONFORMARE.md` | ≥ 1 match referencing this stone |

## Phase B — conformare re-cast (orchestrator-cast)

**Bar: conformare spell re-cast on `src/types.rs` returns CONFORMANT for TypeError.** The spell's audit reports:
- Outer struct field declares span at type level — `err.span` is universal single-path access
- Kind enum variants carry no span fields — discipline can't be silently broken by future variant authors
- CyclicSubtype rune cites domain rationale; emitter passes Span::unknown() explicitly
- From impls preserve span at conversion boundary
- The Pattern A shape per CONFORMARE.md is structurally present, not just conventional

Re-cast protocol:
- R0 baseline: cast on the post-strike `src/types.rs`
- For any L1/L2 finding: per-finding fix (per `feedback_runes_illegal_when_solvable` — runes only when unsolvable or perf-impairing; otherwise FIX)
- R1+ until L1+L2=0 on TypeError specifically (other error types remain L1 — that's the next stones' scope)

## Calibration

**Phase A: 60-120 min Mode A.** Cascade is mechanical (114+ emitters, ~30+ consumers). Substrate-as-teacher discipline handles the cascade via Rust compile errors. Comparable to arc 241 stones 241.8 (27 files / 41 min) and 241.9 (33 files / 50 min) — larger emitter count here but per-site rewrite is more mechanical (relocate span field, no shape redesign per variant).

**Phase B: NO CAP.** Conformare re-cast determines convergence. The spell's audit confirms TypeError conforms structurally; other error types remain L1 (not this stone's scope).

## Pre-spawn baseline checks (verified at HEAD)

1. Lib: **890 PASS / 0 FAIL**
2. tests/function: 8/0 PASS
3. All Stone 243.x probes preserved (only probe_arc243_stone3 added; pre-stone it FAILS to compile per FM 2-bis design)
4. Clippy: **897** (post-Stone-241.18a baseline)
5. `src/types.rs` HEAD: `pub enum TypeError` flat shape; 15 of 16 variants have span; CyclicSubtype lacks span; 16-arm match in parse.rs:154-172 extracts span via exhaustive destructure
6. Conformare spell first-cast verdict: L1 finding on TypeError::CyclicSubtype; recommended starter for retrofit

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | TypeError struct's Display impl needs `self.kind` access not `self` | Compile errors at Display | substrate-as-teacher: mechanical update |
| **T2** | Tests that destructure TypeError variants directly (e.g., probe_arc237_stone1) need shape updates | Test compile failures | mechanical per-site update |
| **T3** | From<ArgSpecError> for TypeError (if exists) — ensure ArgSpecError's classify() span feeds new struct field | Code audit during cascade | check impl; ensure `span: classify().0` not `span: Span::unknown()` |
| **T4** | Some emitters may have complex span expressions that need careful relocation | Per-emitter audit during sweep | every emitter currently has span in scope at construction; just move to outer struct field |
| **T5** | The 16-arm match collapse to `err.span` is THE load-bearing UX win — verify the simplification lands cleanly | Manual review of parse.rs:154-172 post-strike | should be ~6 lines vs current ~17 |
| **T6** | Sub-stone scope creep (sonnet starts retrofitting other error types) | Post-strike audit | only TypeError; other types are next stones' scope per spell's ordering |
| **T7** | New deferral language sneaks in (e.g., "future arc could collapse `span_prefix` 5x helper too") | Exigere preflight grep | strip; no deferral comments allowed |

## What completion looks like

### Phase A — substrate refactor verified

- All 10 scorecard rows green
- All 12 structural verification rows green
- Lib + workspace test-build green
- FM 2-bis probe 3/0
- SCORE Phase A authored

### Phase B — conformare re-cast convergence

- Orchestrator casts conformare on src/types.rs
- Spell reports CONFORMANT for TypeError specifically
- Other error types remain L1 (next stones' scope; explicitly out of THIS stone's bar)
- SCORE Phase B section authored by orchestrator post-cast

### Phase C — commit + push (orchestrator-direct, atomic)

- Atomic commit covers: src/types.rs (refactor) + every cascade site + CONFORMARE.md update + SCORE doc + the probe was already committed STRIKE-READY (separate prior commit OR same atomic per cleanup preference)
- Push to arc-170-gap-j-v5-deadlock-state
- Stone 243.4 opens next (per spell's ordering: ParseStep::ArityMismatch retrofit + parser-API head_span threading)

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.8 | defstruct HARD CUT + 27-file cascade | +864/-644 | 60-120 min | ~41 min UNDER band |
| 241.9 | defenum HARD CUT + 33-file cascade | +809/-576 | 60-120 min | ~50 min UNDER band |
| 241.10 | src/remedy/ mint + ranked-remedy schema HARD CUT + 160-site cascade | substantial mixed | 120-180 min | two-session ship |
| 241.11 | define HARD CUT + 271-site cascade | +7957/-9158 net -1201 | 120-240 min | ~98 min UNDER band |
| 241.18a Phase A | mint src/function/ + migrate 5 fns + cascade | +519 net | 90-150 min | ~30-60 min |
| **243.3 (this)** | **TypeError Pattern A + ~114 emitter cascade + ~30+ consumer cascade + Display + From impls** | **TBD (probably net flat — variants lose span field, struct gains it)** | **A: 60-120 min; B: no cap** | **TBD** |

## What this stone unblocks

**Stone 243.4** — ParseStep::ArityMismatch retrofit + parser-API sister-walk for head_span threading. The TypeError pattern proven here applies directly to ParseStep (smaller scope; ~30 sites).

**The spell's grimoire-earning second proof** — after Stone 243.3 lands, conformare cast on src/types.rs returns CONFORMANT; the spell has now (a) predicted the pattern (first cast) and (b) verified its successful application (post-retrofit cast). Two-cycle validation.

**Future error-type retrofits** — Pattern A applied to TypeError is the template; subsequent stones (RuntimeError, CheckError, etc.) apply the same shape mechanically.
