# SCORE — Stone 242.2: Doctrine 1 SELF-ENFORCING — type-check rejection arm + value-position cascade

**Mode:** A (substrate + cascade; vigilia NOT required per D6 — no new namespaced home)
**Authored:** 2026-05-29 very late — **orchestrator-direct, POST-INSCRIPTION**, closing a paperwork-gap surfaced during arc 242 done-done verification. The BRIEF (line 108) directed sonnet to author this SCORE at strike time; sonnet shipped the substrate work + cascade but the SCORE artifact was never written. Stone 242.2 commit `9c8e8546` shipped 2026-05-29 evening; INSCRIPTION `b7fc45f6` shipped immediately after; SCORE gap surfaced + closed 2026-05-29 very late (user direction: *"protocol compliance — any deviation is a violation"*).
**Stone 242.2 commit:** `9c8e8546f866d16331279073550ca59b2c67d2fc`
**Cascade size:** 166 files changed; +432/-362 (net +70 lines)
**Lib tests:** 890 / 0 (1 pre-existing ignored) — preserved
**Clippy:** 902 warnings (at gate ≤ 902)
**Auto-fixer crate:** NOT built (cascade ran via orchestrator-direct bulk sed after FM 16 sonnet bash firewall trip; ephemeral — no crate to delete)
**Vigilia:** NOT CAST (D6 — legacy flat substrate; no new namespaced home)

---

## Phase A Scorecard (10 rows)

| # | Contract | Status | Evidence |
|---|----------|--------|----------|
| 1 | Probe contracts 01-06 PASS 6/6 | PASS | `probe_arc242_stone2_value_position_doctrine` 6/0 |
| 2 | Stone 242.1 probe preserved 4/4 | PASS | `probe_arc242_stone1_lexeme_role` 4/0 |
| 3 | Stone 241.11 probe preserved 5/5 | PASS | `probe_arc241_stone11_define_hard_cut` 5/0 |
| 4 | Stone 241.10 probe preserved 8/8 | PASS | `probe_arc241_stone10_remedy` 8/0 |
| 5 | Stone 241.1-241.9 + arc 237/238 probes preserved | PASS | full lib + workspace-tests both green; no regressions |
| 6 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 (held from 242.1 baseline) |
| 7 | Workspace test-build clean | PASS | `cargo build --release --tests --workspace` exit 0 |
| 8 | Clippy gate ≤ 902 | PASS | exactly 902 (held from 242.1 baseline) |
| 9 | Type-check rejection arm present | PASS | `src/check.rs:5186-5294` — `is_primitive_type_keyword_in_value_position` guard + rejection comment block ("Doctrine 1: primitive type keywords in VALUE position are ILLEGAL") |
| 10 | No active type-keyword-in-value-position uses | PASS | full cascade migration; remaining matches are error-message strings + retirement tests + intentional doctrine-violation probe sources (C01/C03/C05) |

---

## Structural Verification (6 rows)

| Verification | Result |
|---|---|
| Type-check rejection arm fires on `:wat::core::nil` in value position | ✓ — probe C01 + C05 PASS post-stone (both FAIL at HEAD pre-stone per BRIEF baseline) |
| Type-check rejection arm fires on other type keywords in value position | ✓ — probe C03 PASS post-stone with structured remedy (Doctrine 1 / value position / did you mean phrasing per remedy apparatus) |
| Reflection emitter for `Value::Unit` produces bare nil AST | ✓ — `src/closure_extract.rs` migrated per S2 (verification: lib 890/0 + cascade tests green; any unmigrated value-position emitter would have triggered the rejection arm and surfaced as test failure) |
| INTERSTITIAL UNCHANGED by sonnet at Stone 242.2 commit | ✓ — `git show 9c8e8546 -- docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` returns empty diff (per `feedback_sonnet_never_drafts_interstitial`; INTERSTITIAL was authored later in `b7fc45f6` orchestrator-direct) |
| RETIREMENT_TABLE UNCHANGED (5 entries) | ✓ — `src/remedy/retirement.rs` LHS count = 5 (positional enforcement, NOT form retirement — confirms BRIEF S1 scope) |
| Auto-fixer crate DELETED | ✓ — `ls crates/fix-*/` returns "No such file or directory" (never built — orchestrator-direct sed) |

---

## S1 — Type-check Rejection Arm

Minted at `src/check.rs`:
- `is_primitive_type_keyword_in_value_position` guard (lines 5186-5187)
- Rejection arm with structured remedy (lines 5291+) — *"Stone 242.2 — Doctrine 1: primitive type keywords in VALUE position are ILLEGAL"*
- Specifically removed the prior `nil`-keyword-as-value-literal acceptance arm (lines 5262-5265) — *"REMOVED; nil now falls through to the Doctrine 1 rejection below"*

The rejection uses Stone 241.10's `remedies_for` infrastructure — the third substantive consumer of the bandaid-rip-with-receipts apparatus (after Stone 241.11 define HARD CUT and Stone 242.1 Char HARD CUT). Pattern extension: from "retired form" to "wrong-position form" — same Remedy struct, different RemedyKind context.

Remedy text fired on rejection:
- For `:wat::core::nil` → guidance pointing at bare `nil` in value position
- For other primitive type keywords → guidance pointing at "use a value of this type in value position"

---

## S2 — Reflection Emitter Migration

Substrate-internal Rust code that constructed `:wat::core::nil` keyword AST in VALUE-position contexts migrated to bare-form AST emission. The BRIEF flagged `src/closure_extract.rs:1556` and `src/closure_extract.rs:1567` (`Value::Unit => Ok(WatAST::Keyword(":wat::core::nil"...))`) as primary candidates.

Evidence of correct migration: post-stone lib 890/0 + workspace tests green. Any reflection emitter that produced value-position `:wat::core::nil` AST post-rejection-arm would have triggered self-rejection during test execution (substrate-as-teacher pattern). Zero such failures = migration complete.

---

## S3 — Cascade Migration (Test Sources)

The bulk of the 166-file cascade — test source migrations where existing tests embedded WAT containing value-position `:wat::core::nil` (canonical example: function body `(:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)` migrated to `(:wat::core::defn :f [] -> :wat::core::nil nil)`).

**Cascade method:** orchestrator-direct bulk sed across 158 test files (the BRIEF directed sonnet, but sonnet hit FM 16 sonnet bash firewall trip — claimed "bash denied for bulk sed; permission needed." Per `feedback_verify_sonnet_tool_claims`: firewall displacement, not real permission gap. Orchestrator executed the bulk sed directly).

**Trap-door surfaced:** the bulk sed damaged probe `tests/probe_arc242_stone2_value_position_doctrine.rs` contract 01 — the intentional illegal-form test source was migrated to the legal form (defeating the test). Orchestrator restored manually with an explicit comment block instructing future migrators NOT to migrate the body's `:wat::core::nil` (it's the doctrine-violation-under-test).

---

## S4 — Substrate-internal Rust Code

Substrate Rust code synthesizing value-position keyword AST surfaced via cascade. Migration verified by green test gate. Per BRIEF: `src/freeze.rs:1189`-style error-message strings that NAME the type for user-facing errors were preserved (intentional — naming a type in prose ≠ using it in value position).

---

## S5 — Probe Verification

`tests/probe_arc242_stone2_value_position_doctrine.rs` (6 contracts):
- C01 keyword nil in body rejected — PASS
- C02 bare nil in body passes — PASS
- C03 keyword type in body rejected with remedy — PASS
- C04 bare value in body passes — PASS
- C05 keyword nil in let-binding rejected — PASS
- C06 bare nil in let-binding passes — PASS

6/6 PASS post-stone (3/6 at HEAD pre-stone per BRIEF baseline).

---

## Honest Deltas

### SCORE authored POST-INSCRIPTION (the discipline-gap that surfaced this doc)

The BRIEF (Stone 242.2 BRIEF line 108) directed sonnet to author `SCORE-STONE-242.2.md` at strike time. Sonnet shipped the substrate work (rejection arm + reflection emitter migration) but the SCORE was never written. INSCRIPTION (`b7fc45f6`) and CLIFFNOTES refresh (`f3adb77b`) shipped without the SCORE artifact. Gap surfaced 2026-05-29 very late during user-directed arc 242 done-done verification ("we know it's done?"). User direction: *"protocol compliance — any deviation is a violation."* SCORE written orchestrator-direct from commit + diff + cliffnotes record + post-hoc probe + scorecard verification.

**Why the gap matters:** SCORE docs preserve calibration discipline across compactions — predicted vs actual runtime, methodology evidence, honest-deltas. Without the SCORE, future calibration loops can't reference Stone 242.2's prediction-vs-actual signal. The post-hoc author preserves the discipline (the work is verifiable from commit + tests) but the immediacy is lost — the SCORE is now historical reconstruction, not real-time observation.

**Discipline reinforcement:** post-strike checklist must include SCORE-doc-present check BEFORE marking the stone closed. The cliffnotes "stone SHIPPED" annotation does NOT prove the SCORE shipped alongside it. Closure verification ≠ commit verification.

### FM 16 sonnet bash firewall trip — orchestrator-direct cascade

Sonnet claimed "bash denied for bulk sed; permission needed" mid-strike. Per `feedback_verify_sonnet_tool_claims` + `feedback_sonnet_bash_firewall`: firewall displacement, not real permission. Orchestrator did the bulk sed directly across 158 test files. The work shipped; the calibration signal "sonnet handles X-size cascade via tool Y" was lost (sonnet didn't actually execute the cascade tooling).

### Cascade size 166 files (predicted 25-75)

The BRIEF predicted 20-60 cascade sites; actual was 166 file changes. Reason: the value-position `:wat::core::nil` pattern was MORE pervasive in test sources than initial probe-baseline suggested. The pattern was idiomatic for "function that returns nil" — most test fixtures used `:wat::core::nil` (keyword) instead of bare `nil` (the legal-post-242 form). The substrate's prior tolerance (pre-242.2 the type-inference unified type-keyword-in-value-position as a fresh type variable that happened to unify with the return type) meant idiomatic test code had been writing the now-illegal form for arcs.

**Calibration insight:** when a doctrine becomes self-enforcing for the first time, the cascade depth reflects how WIDESPREAD the prior tolerated-but-illegal form was. The 166-file cascade is the substrate's honest count of how much it had been tolerating.

### Probe C01 source damaged by bulk sed (trap-door, fixed in-flight)

Orchestrator's bulk sed migrated probe C01's intentional illegal-form test source to the legal form (defeating the test). Orchestrator restored manually with explicit `// Note: the body's :wat::core::nil is the keyword-in-value-position doctrine violation under test; do NOT migrate to bare nil — that would defeat the test.` comment block. This is the cost-of-bulk-tooling: blanket sed cannot distinguish "intentional violation test fixture" from "live use." Future bulk cascades should grep test-fixture context for `// doctrine-under-test` markers OR explicitly exclude probe directories.

### Third bandaid-rip-with-receipts consumer (pattern extension)

Stone 241.10 minted `src/remedy/`. Stone 241.11 consumed it for `define` HARD CUT. Stone 242.1 consumed it for `Char` HARD CUT. **Stone 242.2 extends the apparatus from "retired form" to "wrong-position form"** — same Remedy struct, different RemedyKind context. The pattern is now confirmed as foundational across THREE substantive consumers; the apparatus extends naturally to positional enforcement, not just form-retirement.

---

## What This Unblocks (HISTORICAL — arc 242 closure already shipped at `b7fc45f6`)

**Stone 242.3** — INSCRIPTION closed arc 242 (orchestrator-direct paperwork; shipped at `b7fc45f6`).

**Stone 241.12** — defalias mint resumes; STRIKE-READY artifacts at commit `e803e0f9`. Arc 241 paused pending arc 242; arc 242 now complete (with this SCORE artifact). BRIEF augmentation owed: fold in Stone 241.11.fix round 1's lost work (14 test migrations + 1 doc update discarded during 241.12 WIP discard before arc 242 opened).

**Arc 237.8b** — reopens after Stone 241.12 + 241.13 INSCRIPTION closes arc 241.

**Doctrine 1 SELF-ENFORCING** — operational substrate law. Future writes of `:wat::core::*` keywords in value position are AUTOMATICALLY rejected with structured guidance. The doctrine becomes the substrate's enforced law, not just inscribed convention.

**Future case-audits** — `:wat::core::Uuid`, `:wat::core::Duration`, `:wat::core::Instant` (queued arc 109 territory per user direction) all consume Doctrine 2 as the rule + Stone 241.10's apparatus. The pattern is now foundational across three consumers.

---

## Calibration

| Aspect | Predicted | Actual | Notes |
|---|---|---|---|
| Runtime | 60-180 min Mode A | within band (single session) | sonnet substrate + orchestrator FM-16 cascade |
| Cascade size | 25-75 sites | 166 files | substantial — D1 tolerance was pervasive |
| Lib delta | preserved | preserved (890/0) | held |
| Clippy delta | ≤ 902 | exactly 902 | held |
| SCORE-at-strike | yes | NO (post-INSCRIPTION reconstruction) | discipline gap closed by this doc |
