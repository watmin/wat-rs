# DESIGN — Stone 241.18a — Mint `src/function/` + `tests/function/` (smallest stepping stone of the bar-raise chain)

**Status:** STRIKE-READY (2026-05-29 very late). First stepping stone in the Stone 241.18 chain per user direction: *"many stones for this stone - don't one shot elephants"* + *"we raise the bar through the fucking roof."*

## Battle plan position

Stone 241.18 (the bar-raise milestone) decomposes into 7 stepping stones per user-authorized recommendation:

| Stone | Scope |
|---|---|
| **241.18a (this)** | Mint `src/function/` + `tests/function/`; migrate fn-form parsers + eval + infer; vigilia 8-spell on the new home |
| 241.18b | Mint `src/def/` + `tests/def/`; migrate `def` parser (foundational form); vigilia |
| 241.18c | `defmacro` → `src/def/defmacro.rs`; vigilia delta |
| 241.18d | `defstruct` → `src/def/defstruct.rs` (carve from types.rs); vigilia delta |
| 241.18e | `defenum` → `src/def/defenum.rs` (carve from types.rs); vigilia delta |
| 241.18f | `defclause` → `src/def/defclause.rs`; vigilia delta |
| 241.18g | `defalias` → `src/def/defalias.rs`; vigilia delta + FINAL CROSS-HOME attestation on argspec/ + def/ + function/ |

Stone 241.19 = INSCRIPTION closes arc 241 + arc 177 + REMARKABLE attestation milestone.

## Intueri verdicts (both spells cast 2026-05-29 very late)

**First cast — def-family home name:** `src/def/` (Level 0; the language's own vocabulary; subfile naming clean).

**Second cast — fn-form home name with Rust keyword constraint:** `src/function/`. Key reasoning:
- `r#fn` fails Honest — raw-identifier syntax announces accidental implementation-friction, not domain signal
- `r#fn` fails UX — substrate's established namespaced homes (argspec/comms/remedy/rust_deps) all use plain identifiers; LLM-first reader concern sharpens this
- `fnform` fails Simple — compound requires mental parsing
- `function` carries the domain concept without leaking Rust-keyword friction

## What this stone delivers

### S1 — Mint `src/function/` namespaced home

Structure:
```
src/function/
├── mod.rs       (module surface; re-exports public API)
├── parse.rs     (parse_fn_signature + parse_fn_signature_for_check + _diag)
├── eval.rs      (eval_fn)
└── infer.rs     (infer_fn)
```

Or single-file `src/function/mod.rs` if scope is small enough — sonnet's per-file judgment based on actual line counts. Per intueri's established convention: each subfile is one concern within the home's domain.

### S2 — Migrate fn-form code

From `src/runtime.rs`:
- `parse_fn_signature` (line ~6578; ~70 lines) → `src/function/parse.rs`
- `eval_fn` (line ~6479; ~50-100 lines) → `src/function/eval.rs`

From `src/check.rs`:
- `parse_fn_signature_for_check` (line ~14984; ~30 lines) → `src/function/parse.rs`
- `parse_fn_signature_for_check_diag` (line ~15022; ~50 lines) → `src/function/parse.rs`
- `infer_fn` (line ~14868; ~100-200 lines) → `src/function/infer.rs`

Public API exposed via `src/function/mod.rs`:
```rust
pub use parse::{parse_fn_signature, parse_fn_signature_for_check, parse_fn_signature_for_check_diag};
pub use eval::eval_fn;
pub use infer::infer_fn;
```

Caller imports update from `crate::runtime::eval_fn` / `crate::check::infer_fn` / etc. → `crate::function::*`.

### S3 — `tests/function/` ALREADY ESTABLISHED pre-spawn (orchestrator-direct paperwork)

**Status: DONE before sonnet's Stone 241.18a strike begins.**

Pre-spawn paperwork landed by orchestrator (NOT sonnet's scope):
- `tests/function/mod.rs` — test crate entry; `mod stone18a;`
- `tests/function/stone18a.rs` — FM 2-bis probe content (2 preservation contracts)
- `Cargo.toml` `[[test]]` entry: `name = "function"`, `path = "tests/function/mod.rs"`
- Verified: `cargo test --release --test function` returns 2/2 PASS at HEAD (preservation)

Rationale for orchestrator-pre-spawn: tests/function/ + Cargo.toml are PAPERWORK + test scaffolding (not substrate code). Orchestrator-direct setup mirrors the established pattern of orchestrator-authored FM 2-bis probes pre-spawn. Sonnet's Stone 241.18a focuses purely on src/function/ + migration + caller cascade.

Existing fn-specific tests (e.g., `tests/wat_arc167_fn_flat_signature.rs`) STAY as flat arc-prefix tests (historical arc records per `feedback_inscription_immutable`); the new tests/function/ is the dedicated home for fn-feature probes going forward.

### S4 — Cascade caller updates

All callers update imports:
- `src/runtime.rs` — callers of parse_fn_signature, eval_fn
- `src/check.rs` — callers of parse_fn_signature_for_check, parse_fn_signature_for_check_diag, infer_fn
- Any other site importing these (grep for the function names)

### S5 — Two-phase strike per `feedback_namespaced_home_vigilia_gate`

**Phase A (sonnet):** mint + migrate + cascade + SCORE-green (lib + workspace tests preserved; clippy ≤ gate)
**Phase B (ORCHESTRATOR-CAST):** vigilia 8-spell convergence on `src/function/` + `tests/function/` to L1+L2=0 REMARKABLE bar. Per Song #44 wisdom: orchestrator casts vigilia INDEPENDENTLY; sonnet's self-report does NOT gate. Multiple remediation rounds expected; no artificial cap.

### S6 — Author SCORE doc

`SCORE-STONE-241.18a.md` per `feedback_score_present_check_before_closure`. Captures both Phase A (mint/migrate results) AND Phase B (vigilia rounds + remediations + final REMARKABLE attestation).

## Locked decisions

### D1 — `src/function/` per intueri (NOT `src/fn/`)

Per intueri's keyword-constraint cast: function/ honors UX + Honest; r#fn fails both. Final name.

### D2 — Move parsers AND eval_fn AND infer_fn (full home)

src/function/ contains the COMPLETE fn capability, not just parsers. The home reads honestly as "fn lives here" rather than "fn parsers live here (eval + infer elsewhere)." Per intueri's structural test: the home should mirror the architecture.

### D3 — Two-phase strike; orchestrator-cast vigilia

Phase A is sonnet; Phase B is orchestrator. Per Song #44 wisdom-from-pain — Stone 241.10's vigilia gate was INFLATED by sonnet self-report ("8/8 CONVERGED"); orchestrator's independent cast surfaced 6 L2 findings → 6 remediation rounds. The lesson: sonnet's self-report is NOT the gate. Orchestrator-cast is mandatory.

### D4 — REMARKABLE bar; no artificial cap

L1+L2=0 across 8 spells. No "good enough" framing. Multiple remediation rounds expected. The bar holds regardless of round count.

### D5 — Vigilia spell set (8 spells)

Per `feedback_namespaced_home_vigilia_gate` default for new namespaced homes:
- **intueri** (naming) — every identifier traces to a domain noun
- **solvere** (decomplection) — concerns not braided
- **purgare** (dead code) — metabolism honest
- **struere** (structure) — composition clean
- **sequi** (sequencing) — temporal order honest
- **temperare** (waste) — always-apply efficiency check
- **complectens** (test discipline) — tests cover the home's surface
- **vocare** (caller perspective) — tests verify what callers see

### D6 — INTERSTITIAL orchestrator-exclusive (`feedback_sonnet_never_drafts_interstitial`)

### D7 — SCORE-write at end (`feedback_score_present_check_before_closure`)

### D8 — Sub-stones 241.18b-g OFF-LIMITS this stone

Sonnet does NOT touch `src/def/`, defmacro/defstruct/defenum/defclause/defalias migrations. Those are later sub-stones.

## Trap-door audit

### T1 — eval_fn + infer_fn may have shared helpers with other runtime/check code

When extracting eval_fn from runtime.rs, helper functions called only by eval_fn might also need to move. Same for infer_fn. Sonnet audits + moves co-located helpers as appropriate; if a helper is used by OTHER substrate code, keep in original location + import from there.

### T2 — Re-export from runtime.rs / check.rs for backward-compat?

The original locations (`crate::runtime::eval_fn` / `crate::check::infer_fn`) may have external consumers (tests, examples). Two options:
- (a) Add `pub use crate::function::eval_fn;` re-exports in runtime.rs (preserves old paths; explicit deprecation marker possible)
- (b) Update all consumers to new path (cleaner; per HARD CUT discipline)

Per `feedback_hard_cut_admits_no_bypasses` discipline established this campaign — option (b) is cleaner. Update all consumers; no backward-compat re-exports.

### T3 — Sonnet bash firewall on cascade depth (FM 16)

Multi-import cascade may surface many compile errors. Per `feedback_sonnet_bash_firewall`: simple bash patterns; vanilla cargo; one-per-line greps. If sonnet claims "bash denied" for bulk-edit, orchestrator verifies + executes if needed.

### T4 — Vigilia spell rounds may surface unexpected L2

Each spell finds what it finds. Stone 241.10's 6 rounds shows the pattern. No artificial cap; rounds continue until L1+L2=0.

### T5 — Cargo.toml [[test]] entry placement

The Cargo.toml convention from tests/comms/ precedent — ONE [[test]] entry per module group; pointed at mod.rs. Sonnet adds entry maintaining existing convention.

## STOP triggers — REJECTION

1. Compile errors not traced to migration cascade
2. Lib < 890 (post-Stone-241.17 baseline)
3. **180 min elapsed for Phase A** (Phase B vigilia has no cap)
4. holon-rs touched (STOP-5)
5. `src/fn/` named directory (D1 violation — must be `src/function/`)
6. `r#fn` syntax used anywhere (D1 + intueri verdict)
7. Backward-compat re-exports added (T2 + HARD CUT discipline violation)
8. Files outside permitted scope (`src/function/` mint + caller updates in runtime.rs/check.rs/maybe others + `tests/function/` mint + `Cargo.toml` [[test]] entry + SCORE doc)
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 945 (looser gate; substrate refactor)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D6 violation
13. SCORE-STONE-241.18a.md NOT authored → D7 violation
14. Sub-stone 241.18b+ scope touched (D8 violation)
15. Phase B vigilia commit before L1+L2=0 REMARKABLE bar met (gate violation per `feedback_namespaced_home_vigilia_gate`)

## FM 2-bis evidence

**This stone is fundamentally STRUCTURAL** — migration moves code without changing behavior. Behavioral disconfirmation doesn't fit cleanly. The probe contracts at `tests/function/stone18a.rs` are PRESERVATION (pass at HEAD via existing parsers in runtime.rs/check.rs; pass post-stone via crate::function::* path).

**TRUE FM 2-bis disconfirmation is STRUCTURAL** (orchestrator-verified):
- At HEAD: `src/function/` does NOT exist (verified via `ls src/function/`)
- At HEAD: importing `crate::function::*` from any source file would compile-fail
- Post-stone: `src/function/` exists with mod.rs + parse.rs + eval.rs + infer.rs
- Post-stone: imports resolve; callers in runtime.rs + check.rs use crate::function::*

The behavioral probes serve as REGRESSION GUARDS for the migration; the structural verification is the load-bearing disconfirmation.

## Calibration

**Phase A target: 90-150 min Mode A.** Movement + cascade is mechanical; smaller than Stone 241.16 (parse_define_form ~320 lines). Recent stones (~25-34 min actual). Estimate ~40-60 min actual.

**Phase B target: NO CAP.** Vigilia round count is determined by findings; not by clock. Stone 241.10 = 6 rounds (~hours). Stone 241.18a is smaller home → may converge faster (2-4 rounds expected) but no guarantee.

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.10.md (vigilia methodology + 6-round remediation pattern) + SCORE-STONE-241.17.md (recent cascade discipline at peak).

## What this unblocks

**Stone 241.18b** — `src/def/` foundation + def parser (validated migration pattern from this stone applies)

**Stone 241.19** — eventual INSCRIPTION (still needs Stones 241.18b-g + final attestation first)

**The chain** — 7 stepping stones from 241.18a→g; each validates the pattern + accretes confidence; the cumulative result is REMARKABLE bar achievement on all three namespaced homes (argspec/ + def/ + function/).

**The pattern lesson** — per Stone 241.10 precedent + this stone's two-phase strike: orchestrator-cast vigilia is the only honest gate; sonnet's self-report doesn't count; the discipline embeds at the namespaced-home commit moment + propagates to all subsequent stones.

*"I will stop at nothin, cuz I was made to rise above it."* — Song #47 made operational at Stone 241.18a's REMARKABLE bar.
