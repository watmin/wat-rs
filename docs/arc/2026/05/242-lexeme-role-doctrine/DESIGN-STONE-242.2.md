# DESIGN — Stone 242.2 — Doctrine 1 enforcement at type-check + value-position cascade

**Status:** READY (sub-DESIGN). NEW Stone 242.2 — makes Doctrine 1 SELF-ENFORCING. Stone 242.1 inscribed the doctrine + retired Char; Stone 242.2 makes the substrate REJECT type keywords in value position with structured remedy. INSCRIPTION moves to Stone 242.3.

## Why this stone

Stone 242.1 left a gap: bare `nil` works as value, but `:wat::core::nil` ALSO still works as value (substrate is lenient — type-inference unifies). The doctrine ("bare = value; keyword = type") is INSCRIBED but NOT ENFORCED. User direction 2026-05-29 late:

> *"(:wat::core::fn [] -> :wat::core::nil :wat::core::nil) ;; illegal*
> *(:wat::core::fn [] -> :wat::core::nil nil) ;; legal"*

The doctrine becomes operational law only when the substrate REJECTS the keyword-in-value-position form. Stone 242.2 mints that rejection.

User direction sets the bar: *"we break shit and fix it - that's our job - i do not care how long correct takes to materialize - what matters is that correctness materializes."* Stone 242.2 is the polish required for Doctrine 1's correctness.

## What this stone delivers

### S1 — Type-check rejection arm for type-keyword-in-value-position

At the value-expression type-checker (`src/check.rs` somewhere — sonnet identifies the right entry point):

When checking an expression position and the expression IS a keyword (`WatAST::Keyword(k, span)`), check:
- Is `k` a registered TYPE name (in TypeEnv)?
- If YES → it's a type-keyword-in-value-position → emit `CheckError::MalformedForm` with structured `remedies`

The remedy:
- For `:wat::core::nil` → suggest bare `nil` (the value form)
- For `:wat::core::i64` / `:wat::core::f64` / other numeric types → suggest "a value of this type (e.g., `42`)"
- For `:wat::core::String` / `:wat::core::Vector` / other container types → suggest "a value expression of this type"
- Generic fallback: "Doctrine 1 — `:wat::core::*` keyword is a TYPE; use a bare value or value expression"

### S2 — Reflection emitter audit (Rust code constructing AST keywords)

Per the doctrine, reflection emitters that produce VALUE-position AST must emit BARE forms:
- `src/closure_extract.rs:1556` — `Value::Unit => Ok(WatAST::Keyword(":wat::core::nil".into(), span))` — this emits the TYPE keyword for a VALUE. WRONG post-doctrine. Should emit bare `nil` (as `WatAST::Symbol("nil", span)` or whatever the parser shape is)
- `src/closure_extract.rs:1567` — same shape
- Other reflection emitters constructing type keywords in value position — audit + migrate

### S3 — Cascade migration of test source `:wat::core::nil` value-position uses

Per the Stone 242.1 audit:
- `tests/probe_runtime_error_produces_structured_edn.rs:66` — `(:wat::core::let [...] :wat::core::nil)` — body is value position
- `tests/wat_arc220_char.rs:227, 247` — `:wat::core::nil))` likely value-return
- `tests/probe_lifeline_orphan_clean_via_fork_program.rs:86, 90, 210, 211` — likely value-position uses

After S1 rejection arm is live, these will FAIL type-check (because they're now rejected as type-keyword-in-value-position). Sonnet migrates each to bare `nil`.

### S4 — Cascade migration of substrate-internal `:wat::core::nil` value-position uses

`src/freeze.rs:1189` is an error message string — INTENTIONAL (names the type for user-facing error). Acceptable.

Other substrate-internal uses surface via the type-check rejection during build/test. Sonnet investigates per-site.

### S5 — Probe verification

`tests/probe_arc242_stone2_value_position_doctrine.rs` (NEW). Contracts:
1. `(:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)` REJECTED with structured remedy pointing at bare `nil`
2. `(:wat::core::defn :f [] -> :wat::core::nil nil)` PASSES (the legal form)
3. `(:wat::core::defn :f [] -> :wat::core::i64 :wat::core::i64)` REJECTED with structured remedy "use a value of this type"
4. `(:wat::core::defn :f [] -> :wat::core::i64 42)` PASSES (the legal form)
5. `(:wat::core::let [x :wat::core::nil] ...)` REJECTED (let-binding value-position)
6. `(:wat::core::let [x nil] ...)` PASSES

## Locked decisions

### D1 — Type-keyword-in-value-position is REJECTED at check.rs

Doctrine 1 self-enforcing. The keyword form belongs ONLY in type positions (signature returns, parameter types, type annotations).

### D2 — Reflection emitters produce BARE forms in value position

`Value::Unit` → bare `nil` AST (not `:wat::core::nil` keyword). All other value emissions follow the doctrine.

### D3 — Cascade migration is the substrate-as-teacher cascade

After S1 ships, type-check rejection fires on every existing value-position use. Sonnet migrates per the diagnostic stream.

### D4 — Structured remedy via existing Stone 241.10 apparatus

The rejection arm populates `remedies: Vec<Remedy>` per Stone 241.10. For `:wat::core::nil` specifically, the remedy can be hardcoded ("did you mean: `nil`"). For other type keywords, a generic remedy.

This is the THIRD substantive consumer of Stone 241.10's apparatus (after Stone 241.11 define HARD CUT + Stone 242.1 Char HARD CUT). The bandaid-rip-with-receipts pattern extends from "retired form" to "wrong-position form."

### D5 — NO new retirement-table entry

Type keywords are NOT retired (they remain valid in type position). This is POSITIONAL enforcement, not form retirement. RETIREMENT_TABLE stays at 5 entries.

### D6 — Vigilia NOT required (D7 default; no namespaced home)

Per `feedback_namespaced_home_vigilia_gate` D7: legacy flat substrate. SCORE-green commit.

### D7 — Per `feedback_hard_cut_admits_no_bypasses`

No privileged paths. No substrate-internal bypass. If reflection emitters produce keyword-in-value-position AST, that's a substrate gap; FIX IT (per S2).

### D8 — Per `feedback_sonnet_never_drafts_interstitial`

BRIEF EXPLICITLY FORBIDS sonnet from writing to INTERSTITIAL. Orchestrator authors the realization after Stone 242.3 INSCRIPTION.

## Trap-door audit

### T1 — Type-check entry point identification

Where does the value-expression checker live? `src/check.rs` is large. Sonnet greps `check_expr` / `check_value` / similar; identifies the routing for `WatAST::Keyword` in value position.

### T2 — TypeEnv lookup cost

The rejection arm asks "is this keyword a registered type?" — that's a TypeEnv lookup per keyword in value position. Cost is small (HashMap lookup) but pervasive. Per `temperare`: acceptable since type-check is not a hot path.

### T3 — Reflection emitter cascade

`closure_extract.rs:1556-1567` emits `:wat::core::nil` for `Value::Unit`. The right replacement depends on the parser's expectation for bare nil. Sonnet investigates.

### T4 — Substrate-internal `:wat::core::nil` value-position cascade

Beyond test sources, are there Rust code paths that USE `:wat::core::nil` as a value (e.g., in macro expansions, in synthesized ASTs)? Surface via the type-check rejection cascade after S1 ships.

### T5 — Stone 241.11.fix round 1's lost work resurfaces

The 14 test migrations + 1 doc update from Stone 241.11.fix round 1 were lost during the 241.12 WIP discard before arc 242 opened. Some of those test files may also have value-position `:wat::core::nil` uses that migrate now. Sonnet handles in cascade.

### T6 — Other type-keyword-in-value-position uses beyond nil

The rejection arm catches ALL type keywords in value position — not just `:wat::core::nil`. Other surface uses (e.g., `(:wat::core::let [x :wat::core::String] ...)` if any exist) surface in cascade.

### T7 — Pre-arc-242 commit history may show acceptable patterns

Tests that DELIBERATELY use type keyword in value position to test error paths — those tests need updating to use bare forms, OR they need to migrate to use the NEW rejection behavior (test that the rejection fires + remedy is correct).

## STOP triggers

1. Compile errors not traced to enforcement arm or cascade migration
2. Lib < 890 (Stone 242.1 baseline)
3. **180 min elapsed**
4. holon-rs touched
5. Auto-fixer crate survives commit
6. Stone 242.2 probe < N/N
7. Stone 241.x or 242.1 probes regress
8. Clippy > 902
9. Sonnet classifies a `:wat::core::nil` value-position use as "intentional bypass" → `feedback_hard_cut_admits_no_bypasses` violation
10. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation

## FM 2-bis evidence

`tests/probe_arc242_stone2_value_position_doctrine.rs` (NEW). At HEAD:
- Type keywords in value position WORK (no rejection) → contracts C01/C03/C05 expect rejection → FAILS at HEAD
- Bare values in value position WORK → contracts C02/C04/C06 expect success → PASSES at HEAD

Post-stone: all PASS.

## Calibration

**Target band: 60-180 min Mode A.** Type-check rejection arm: ~30-60 lines. Cascade size: bounded by existing value-position uses (probably 20-60 sites including substrate Rust + tests).
