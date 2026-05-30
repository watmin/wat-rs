# DESIGN — Stone 241.13 — `:wat::core::define-dispatch` HARD CUT + DispatchRegistry scaffolding deletion (Enemy 2 of 3)

**Status:** STRIKE-READY (2026-05-29 very late). Enemy 2 in the define-family death campaign (per Stone 241.12 § Battle plan). Stone 241.14 (Enemy 3 — define eval-time residue) follows; Stone 241.15 INSCRIPTION closes arc 241.

## Context

Stone 241.12 (Enemy 1) shipped at `7244cf43` — `:wat::core::defalias` minted native; `:wat::runtime::define-alias` HARD CUT total. Two trap-doors absorbed (closure_extract retired-form emitter; runtime.rs Gap D name-field overwrite). Stone 241.13 attacks Enemy 2.

User directive 2026-05-29 very late: *"define-alias, define-dispatch, define all die"* + *"this is the fight now - we know we have multiple enemies - we manage the fight - we are the best - we prove it relentlessly."*

## Why this stone (the field state)

`:wat::core::define-dispatch` (arc 146) is the dispatch-by-arity+type entity kind that PRE-DATES `:wat::core::defclause` (Stone 237.2 SHIPPED `bdd9eb6c`). The migration is largely complete:

**Active wat-source callers: ZERO** (verified fresh grep). All ops registered via define-dispatch in wat/core.wat have been evacuated per arc 237.7a/7b/7c:
- `:wat::core::length` → Rust ∀T intrinsic (`infer_length` + `eval_length`)
- `:wat::core::empty?` → Rust ∀T intrinsic (`infer_empty` + `eval_empty`)
- `:wat::core::contains?` → Rust ∀T intrinsic (`infer_contains` + `eval_contains`)
- `:wat::core::get` → Rust ∀T intrinsic (`infer_get` + `eval_get`)
- `:wat::core::conj` → Rust ∀T intrinsic (`infer_conj` + `eval_conj`)
- `:wat::core::assoc` → Rust ∀T intrinsic spanning HashMap + Record (`infer_assoc` + `eval_assoc`)

**Substrate scaffolding still LIVE (even though registry is always empty post-evacuation):**
- `src/dispatch.rs` (445 lines)
- `DispatchRegistry` plumbing across `src/freeze.rs`, `src/check.rs`, `src/runtime.rs`, `src/resolve.rs`
- `src/special_forms.rs:194` registry entry
- `freeze.rs` walker arms (lines 1382, 1422)
- `src/check.rs:5618-5620` — dispatch_registry guard (always empty; dead path)

**Tests still reference define-dispatch:**
- `tests/wat_arc146_dispatch_mechanism.rs` — entire arc 146 acceptance test (obsolete; mechanism retiring)
- `tests/probe_arc237_7a_length_intrinsic.rs` — comment says "works TODAY via define-dispatch" but `length` is now ∀T intrinsic; test may be a STALE regression guard
- `tests/probe_arc237_7b_intrinsic_typing.rs` — comment says "exercises CURRENT (define-dispatch) behavior" but `empty?`/`contains?` now ∀T intrinsics; STALE
- `tests/wat_arc144_uniform_reflection.rs:278-298` — STALE comment + assertion (`empty?` no longer dispatch-registered)
- `tests/probe_declaration_form_lift.rs` — declaration-form lifting test includes define-dispatch as one of the lifted forms
- `tests/probe_def_not_special.rs:259, 283` — uses define-dispatch as test fixture

Per `feedback_hard_cut_admits_no_bypasses`: the substrate cannot carry dead-but-live infrastructure for a retired form. Stone 241.13 deletes the entire DispatchRegistry path.

## What this stone delivers

### S1 — HARD-CUT-rejection arm at `src/check.rs`

Mirror Stone 241.11/241.12 pattern:

```rust
":wat::core::define-dispatch" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.13); use ':wat::core::defclause' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

### S2 — 7th RETIREMENT_TABLE entry

`src/remedy/retirement.rs`:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // ... existing 6 entries ...
    // Stone 241.13 — defclause replaces define-dispatch.
    (":wat::core::define-dispatch",   ":wat::core::defclause"),
];
```

### S3 — DELETE `src/dispatch.rs` entirely (~445 lines)

The entire file (`DispatchRegistry`, `register_dispatch`, `parse_dispatch_form`, etc.) goes. Zero wat-source consumers; the registry is always empty.

### S4 — DELETE DispatchRegistry plumbing across substrate

| File | Lines (approx) | Action |
|---|---|---|
| `src/freeze.rs` | use import + `dispatchs: DispatchRegistry` field + `set_dispatch_registry` calls + `dispatchs()` accessor + step-2 registration walker (~30-50 lines) | DELETE |
| `src/check.rs` | `dispatch_registry: Option<Arc<...>>` field + `dispatch_registry()` method + `dispatch_registry guard at line 5618-5620 + env init (~30 lines) | DELETE |
| `src/runtime.rs` | `dispatch_registry` field on SymbolTable (~5 lines) + DispatchRegistry::new() instantiations + form constructors at lines 13372, 13385, 13427, 13568, 13746 (~30-50 lines) | DELETE |
| `src/resolve.rs:328` | `sym.dispatch_registry()` consultation (~5 lines) | DELETE |
| `src/special_forms.rs:194` | entry: `":wat::core::define-dispatch"` | DELETE |
| `src/freeze.rs:1382, 1422` | walker-arm match cases for `:wat::core::define-dispatch` | DELETE (or keep as part of HARD-CUT arm consolidation) |

### S5 — Test migration / deletion

Per-file judgment:

**`tests/wat_arc146_dispatch_mechanism.rs`** — entire arc 146 acceptance test. The MECHANISM IS BEING RETIRED. DELETE the file (or repurpose to verify the HARD CUT acceptance — preferable per substrate-as-teacher pattern).

**`tests/probe_arc237_7a_length_intrinsic.rs`** — purports to be "behavior regression guard" for length via define-dispatch. Length is now ∀T intrinsic (per arc 237.7a evacuation). Two options: (a) DELETE — the intrinsic path has its own tests; (b) UPDATE — repurpose to test the intrinsic path. Per `feedback_no_pre_existing_excuse`: investigate; if intrinsic tests exist elsewhere, delete; otherwise repurpose.

**`tests/probe_arc237_7b_intrinsic_typing.rs`** — same pattern as 7a, for empty?/contains?/get. Same options.

**`tests/wat_arc144_uniform_reflection.rs`** — STALE comments + assertion `line.contains("define-dispatch")`. Update the assertion to match current reflection behavior (the test likely needs to assert defclause or the ∀T intrinsic naming).

**`tests/probe_declaration_form_lift.rs`** — comprehensive declaration-form lift test. Drop the define-dispatch case (probes 3 + variants); preserve other declaration forms (def/defmacro/newtype/typealias). Update comment listing the lifted forms.

**`tests/probe_def_not_special.rs:259, 283`** — uses define-dispatch as fixture. Migrate to `:wat::core::defclause` fixture (semantically equivalent for the test's purpose: showing that `def` is not a "special" head).

### S6 — Update historical comments

`wat/core.wat:8` and adjacent comments referencing `define-dispatch` as the current mechanism — update to reflect retirement (per `feedback_inscription_immutable`, historical references are OK; STALE current-tense comments are not).

`src/runtime.rs:5711-5715` — comments about "Reborn from define-dispatch (core.wat) to Rust builtin" — these are historical and accurate; KEEP.

`src/check.rs:20437` — comment about "Reborn from define-dispatch (core.wat) to Rust builtin" — same; KEEP.

### S7 — Probe verification

`tests/probe_arc241_stone13_define_dispatch_hard_cut.rs` (NEW). FM 2-bis disconfirming.

### S8 — SCORE doc (mandatory)

Per `feedback_score_present_check_before_closure` — author `SCORE-STONE-241.13.md` at end of strike.

## Locked decisions

### D1 — HARD CUT TOTAL per `feedback_hard_cut_admits_no_bypasses`

No "the registry is empty so it's fine" framing. No "but the infrastructure compiles to dead code" framing. Both are privileged-path framings. DELETE.

### D2 — `:wat::core::defclause` is the replacement

Stone 237.2 shipped defclause. It's the surviving dispatch-by-arity+type entity kind. RETIREMENT_TABLE entry names it as the replacement.

### D3 — `src/dispatch.rs` DELETED entirely

Not "tombstone with deprecation warning"; not "kept for reference"; DELETED. The git history preserves it per `feedback_inscription_immutable`.

### D4 — Per-test judgment for migration vs deletion

Sonnet's per-test audit determines whether each test is migrated to defclause (semantic equivalence) or deleted (obsolete regression guard for retired mechanism). No "test scaffolding stays for completeness" — every test either has a current purpose (migrate) or doesn't (delete).

### D5 — Vigilia NOT required (D7 default per `feedback_namespaced_home_vigilia_gate`)

No new namespaced home. SCORE-green commit.

### D6 — SCORE-write at end (D8 from 241.12; `feedback_score_present_check_before_closure`)

### D7 — INTERSTITIAL orchestrator-exclusive (D9 from 241.12; `feedback_sonnet_never_drafts_interstitial`)

### D8 — Build / fold-in scope strictly bounded to Stone 241.13

Stone 241.14 (Enemy 3 — define eval-time residue) is a SEPARATE stone. Sonnet does NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form`, or freeze.rs test fixtures using `:wat::core::define` for bypass tests. Those are Enemy 3 scope.

## Trap-door audit

### T1 — DispatchRegistry deletion cascades through CheckEnv / SymbolTable

The dispatch_registry field is on CheckEnv + SymbolTable. Removing it triggers cascade through every site that reads/writes the field. Likely 30-50 site cascade in src/.

Resolution: cascade per substrate-as-teacher; each site's removal is mechanical.

### T2 — `infer_dispatch_call` may be referenced by code paths not yet identified

The dispatch_registry guard at check.rs:5618 routes to `infer_dispatch_call`. If other code paths reference the function (e.g., via test fixtures), grep first.

Resolution: grep `infer_dispatch_call` across src/ + tests/. Delete callers.

### T3 — Test deletion may surface other tests that depend on dispatch state

If a test file is deleted and another test file references its constants/helpers, build fails. Cascade check.

Resolution: build cycle after each test deletion.

### T4 — `probe_arc237_7a/7b` may be the only regression coverage for evacuated ops

If the ∀T intrinsic path doesn't have separate test coverage, deleting these tests creates a coverage gap. Verify before deleting.

Resolution: grep for the evacuated op names in other test files. If coverage exists, delete; if not, REPURPOSE (rewrite to test the intrinsic path).

### T5 — `wat_arc146_dispatch_mechanism.rs` deletion vs HARD-CUT acceptance repurpose

DELETE = clean removal; REPURPOSE = turn the test into HARD-CUT acceptance (substrate-as-teacher pattern). Preferable: repurpose 1-2 contracts to test HARD-CUT rejection + remedy quality; delete the rest.

### T6 — `closure_extract.rs` may emit define-dispatch AST for prologue re-freeze

Stone 241.12 surfaced an analogous bug for `:wat::core::define`. If closure_extract.rs emits `:wat::core::define-dispatch` keyword AST anywhere, the prologue re-freeze will HARD-CUT-fail post-S1.

Resolution: grep `Keyword.*define-dispatch` in src/closure_extract.rs (+ src/runtime.rs reflection emitters). Migrate or delete.

### T7 — Sonnet "privileged path" temptation (Stone 241.11.fix round 2 lesson)

Per D1. STOP if surfaces.

## STOP triggers — REJECTION

1. Compile errors not traced to dispatch deletion cascade
2. Lib < 890 (post-241.12 baseline) — note: test deletions reduce count; track expected delta
3. **180 min elapsed** (this stone is bigger than 241.12 due to 445-line file deletion + cascade)
4. holon-rs touched (STOP-5)
5. Substrate `:wat::core::define-dispatch` use classified as "infrastructure stays empty so it's fine" without migration → D1 + `feedback_hard_cut_admits_no_bypasses` violation
6. `src/dispatch.rs` PRESERVED (D3 violation; the file must be DELETED, not kept-as-tombstone)
7. Files outside permitted scope (`src/dispatch.rs` DELETED / `src/check.rs` / `src/freeze.rs` / `src/runtime.rs` / `src/resolve.rs` / `src/special_forms.rs` / `src/remedy/retirement.rs` / `src/closure_extract.rs` if reflection emitters touched / test files in S5 inventory / `tests/probe_arc241_stone13_*` / `wat/core.wat` for historical comment update / SCORE doc)
8. Stone 241.13 probe < N/N (target set after probe written)
9. Stone 241.x or arc 237/238/242 probes regress (except the test files in S5 inventory which are EXPECTED to be migrated/deleted)
10. Clippy > 920 (allows for some slack from line shifts post-deletion; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit (Stone 241.10/241.11 ephemeral discipline)
12. Sonnet writes to INTERSTITIAL → D7 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.13.md NOT authored at end → D6 + `feedback_score_present_check_before_closure` violation
14. Stone 241.14 scope touched (`is_mutation_head` arm + `parse_define_form` + etc.) → D8 violation

## FM 2-bis evidence

`tests/probe_arc241_stone13_define_dispatch_hard_cut.rs` (NEW; written + verified disconfirms at HEAD before BRIEF spawns).

## Calibration

**Target band: 90-180 min Mode A.**

Stone 241.13 is BIGGER than 241.12 in deletion scope (445-line file delete + DispatchRegistry plumbing cascade + ~6 test files to migrate/delete) but SMALLER in synthesis (no native form to mint; no consistency fold-in).

Variables:
- `src/dispatch.rs` deletion + cascade — **~30-45 min**
- DispatchRegistry plumbing deletion across substrate — **~30-45 min**
- Test migration/deletion (per-file judgment) — **~30-45 min**
- HARD CUT arm + RETIREMENT_TABLE append — **~10 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.12.md (cascade discipline + trap-door absorption pattern); SCORE-STONE-241.11.md (HARD CUT mass-cascade discipline).

## What this unblocks

**Stone 241.14** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT)

**Stone 241.15** — INSCRIPTION closes arc 241

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`

**The dispatch entity-kind family** — arc 146's "dispatch by arity + type" mechanism RETIRES; arc 237.2's defclause is the surviving entity kind. The substrate's dispatch story collapses to ONE form per `feedback_wat_llm_first_design` (one canonical path per task).
