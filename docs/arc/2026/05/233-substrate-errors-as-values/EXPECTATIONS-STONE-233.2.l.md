# EXPECTATIONS — Arc 233 Stone 233.2.l — #[wat_value] proc-macro structural seal

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean (wat) | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Compile clean (wat-macros) | `cargo build --release -p wat-macros 2>&1 \| tail -5` | 0 errors |
| 3 | **233.2.l probe FLIPS 0/3 → 3/3** | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -5` | `test result: ok. 3 passed; 0 failed` |
| 4 | **wat-macros tests (incl. trybuild compile-fail fixtures)** | `cargo test --release -p wat-macros 2>&1 \| tail -3` | all pass; trybuild compile-fail fixtures verified |
| 5 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 6 | Stone 233.2.k probe still passes | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | Stone 233.2.j probe still passes | `cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.i eval signature probe still passes | `cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 9 | Stone 233.2.h TrackedValue mint probe still passes | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 10 | Stone 233.1 ValueSnapshot probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 45–90 min Mode A
**Upper bound:** 120 min (STOP-3) — per Stone 233.2.l sub-DESIGN
**Confidence:** medium — focused proc-macro work; pattern lineage from existing `#[wat_dispatch]` in wat-macros; trybuild setup is well-trodden but new to this codebase

**Rationale:**
- Proc-macro implementation in `crates/wat-macros/src/wat_value.rs`: ~30-45 min (syn parsing + detection algorithm + opt-in attr handling + compile_error! diagnostic)
- Export from `crates/wat-macros/src/lib.rs`: ~2 min
- Apply `#[wat_value]` to real `pub enum Value`: ~5 min (verify it compiles)
- trybuild fixture setup + 4-5 compile-fail fixtures: ~15-25 min
- Runtime probe assertions: ~5 min
- Verification cascade + SCORE writing: ~10 min

**Risks:**
- trybuild may need a separate `tests/ui_compile.rs` runner that invokes trybuild; sonnet may pick alternative mechanism (e.g., a custom build.rs or process::Command-based compile-checker) if trybuild doesn't fit cleanly
- syn AST navigation has quirks (variant attrs vs field attrs vs enum-level attrs); the opt-in attribute parsing needs care
- Error message span must point at the offending VARIANT, not the whole enum; sonnet may iterate the diagnostic shape
- If 233.2.k introduced any subtle Value variant the macro inadvertently flags (e.g., a self-referential variant we didn't notice), STOP-1 fires — orchestrator surfaces

## Compile-fail contracts (NOT in runtime probe; trybuild fixtures)

The runtime probe covers contracts 2 + 4 from the sub-DESIGN (container variants pass + opt-in works). The remaining compile-fail contracts live in `crates/wat-macros/tests/ui/`:

| trybuild fixture | Contract | Expected |
|---|---|---|
| `ui_wat_value_rejects_box_self.rs` | Box<Self> field rejected | compile error mentioning "wrapping shape" |
| `ui_wat_value_rejects_arc_self.rs` | Arc<Self> field rejected | compile error mentioning "wrapping shape" |
| `ui_wat_value_rejects_self_direct.rs` | Self field rejected | compile error mentioning "wrapping shape" |
| `ui_wat_value_accepts_opt_in.rs` | Opt-in with reason string compiles | compile success |
| `ui_wat_value_rejects_alias_bypass.rs` | (optional) Aliased Box<Self> rejected OR documented limitation | per Decision 1 of sub-DESIGN |

## Out-of-scope rows (REJECTED)

- Application to HolonAST / WatAST / other enums (separate stones)
- Semantic resolution of type aliases (opt-in covers; documented limitation)
- Lint-level enforcement on USER code (substrate-internal seal only)
- holon-rs touched (STOP-4)
- Enum-level escape hatch (per Decision 2 of sub-DESIGN — REJECTED; defeats structural seal)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to macro work
- STOP-2: baseline regress below 827
- STOP-3: 120 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (other enums, semantic resolution, enum-level escape)
- STOP-7: probe still has failures
- STOP-8: existing arc 233 probes regress
- STOP-9: cascade exceeds time-box — apply partial-state-grading

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- Proc-macro implementation (file, line count, detection algorithm summary)
- wat-macros export changes (lib.rs)
- #[wat_value] application to real Value enum (single line; verify cargo build clean)
- trybuild fixture inventory + per-fixture verification status
- Runtime probe contract verification
- Time breakdown by phase
- Calibration band actual vs predicted (45-90 target; 120 STOP)
- 12-row scorecard with verbatim verification command outputs
- Honest deltas (alias bypass behavior — REJECTED via opt-in vs documented limitation)

## What this unblocks (THE STONE THAT CLOSES THE META-CLASS)

- **arc 233 Stone 233.2 sub-chain complete** — j ✓ → k ✓ → l ✓ (closes the trap-door class at both instance + meta layers)
- **Stone 233.2.e** — AST-derived provenance work proceeds on a fully-sealed substrate
- **Stone 233.3** — Errors-as-EDN extension
- **Stone 233.4** — INSCRIPTION (arc 233 closes)
- **arc 232** — defprotocol resumes against the diagnostic-rich substrate
- **Any future enum that adopts `#[wat_value]`** — HolonAST, WatAST candidates if similar trap-doors surface

## The annihilation (this is THE seal)

After this stone:
- Value::Tracked is GONE (233.2.k)
- Adding any wrapping-style variant to Value is a **compile error** with teaching diagnostic
- Escape hatch requires explicit per-variant `#[wat_value(allow_wrapping = "reason")]` with non-empty reason string
- Future authors who reach for the trap-door pattern hit the seal at the highest possible layer

Per FAILURE-ENGINEERING.md ✅✅✅: the SITUATION that produces the trap-door is structurally impossible to construct in source AND structurally impossible to RE-INTRODUCE at future authoring time.

The walk-impossible is sealed.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.l.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md` — sub-DESIGN (commit `57eced2`)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md` — prerequisite (variant retirement)
- `tests/probe_stone_233_2_l_wat_value_seal.rs` — runtime contracts probe (committed alongside BRIEF spawn)
- `crates/wat-macros/tests/ui/*.rs` — trybuild compile-fail fixtures (sonnet creates)
- `scratch/FAILURE-ENGINEERING.md` — ✅✅✅ doctrine
- `feedback_partial_state_grading` — discipline if STOP-3 fires
