# Stone 237.4 sub-DESIGN — rich `:NoMatchingClause` + `:PostconditionFailed` diagnostics

**Status:** PENDING (sub-DESIGN authored 2026-05-25 night-late post-reboot; FM 2-bis probe + BRIEF + EXPECTATIONS pending).

**Scope:** Promote the TEMPORARY error variants shipped in Stones 237.2 + 237.3 to RICH diagnostics per arc 233.3 EDN-shape. Mint `ClauseAttempt` struct (structured per-clause failure reason). Refine `NoMatchingClauseRuntime` → `NoMatchingClause` (HARD CUT rename; structured attempt list). Refine `PostconditionFailedRuntime` → `PostconditionFailed` (ensure-expr snapshot + dual spans). Clean EDN tags. This is diagnostic-richness work — well-trodden per arc 233's 28-variant precedent; NOT new mechanism.

**Why this stone fourth:** Stones 237.2 + 237.3 shipped defclause + guards/ensures with TEMPORARY error variants explicitly marked "Stone 237.4 refines." Per arc 233 doctrine (errors as teaching values), the diagnostic must surface WHY each clause failed (arity / type / guard) — the current flat `Vec<String>` shows signatures but not failure reasons. Stone 237.4 closes that gap before the migration stones (237.6/7) make defclause the polymorphism backbone.

**Builds on (shipped):**
- Stone 237.2 (bdd9eb6c) — `NoMatchingClauseRuntime` variant + `CheckError::NoMatchingClauseAtCallSite`
- Stone 237.3 (ee5e892c) — `PostconditionFailedRuntime` variant
- arc 233.1 — ValueSnapshot in RuntimeError
- arc 233.2 — Provenance + TrackedValue
- arc 233.3 — 28 RuntimeError variants EDN-serialized (the template this stone mirrors)
- arc 138 — errors carry point-in-code span coordinates

**Out-of-scope (later arc 237 stones):**
- Variadic rest-binder (Stone 237.5)
- Widest-contagion type-checker rule (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)

---

## Current state (diagnosed 2026-05-25 post-reboot)

**`RuntimeError::NoMatchingClauseRuntime`** (`src/runtime.rs:2216`):
```rust
NoMatchingClauseRuntime {
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<String>,   // ← FLAT strings "clause N: (T1, T2)"
    span: Span,
}
```
- Already EDN-serialized (`tagged("NoMatchingClauseRuntime", ...)` at `runtime_error_edn.rs:251`)
- GAP: `attempted_clauses` shows signatures, NOT failure reasons (was the clause skipped for arity? type? guard-false?)

**`RuntimeError::PostconditionFailedRuntime`** (`src/runtime.rs:2231`):
```rust
PostconditionFailedRuntime {
    defclause_name: String,
    clause_index: usize,
    returned_value: ValueSnapshot,
    span: Span,
}
```
- Already EDN-serialized (`runtime_error_edn.rs:267`)
- GAP: no `ensure_expr_snapshot` (which postcondition failed); single span only (no body-vs-ensure distinction)

---

## Locked decisions

### `ClauseAttempt` struct (NEW)

```rust
/// Why a single clause was skipped during defclause dispatch.
/// Stone 237.4 — promotes NoMatchingClause from flat strings to structured
/// per-clause failure reasons (arc 233 errors-as-teaching-values doctrine).
pub struct ClauseAttempt {
    pub clause_index: usize,
    pub declared_arity: usize,
    pub declared_arg_types: Vec<String>,         // formatted TypeExpr per position
    pub failure_reason: ClauseFailureReason,
}

pub enum ClauseFailureReason {
    ArityMismatch { expected: usize, got: usize },
    ArgTypeMismatch { position: usize, expected: String, got: String },
    GuardFalse,                                   // guard evaluated to false
}
```

### `NoMatchingClause` refinement (HARD CUT rename)

```rust
NoMatchingClause {                               // renamed from NoMatchingClauseRuntime
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<ClauseAttempt>,       // ← structured (was Vec<String>)
    span: Span,
}
```

### `PostconditionFailed` refinement (HARD CUT rename)

```rust
PostconditionFailed {                            // renamed from PostconditionFailedRuntime
    defclause_name: String,
    clause_index: usize,
    ensure_expr_snapshot: String,                // ← NEW: the :ensure :fn that failed (rendered)
    returned_value: ValueSnapshot,
    body_span: Span,                             // ← NEW: where the body produced the value
    ensure_span: Span,                           // ← NEW: where the :ensure :fn is declared
}
```

### EDN tags (clean; drop `*Runtime`)

- `#wat.kernel/NoMatchingClause` (was `NoMatchingClauseRuntime`)
- `#wat.kernel/PostconditionFailed` (was `PostconditionFailedRuntime`)
- `ClauseFailureReason` serializes as a tagged sub-value within each attempt

### HARD CUT (per arc 234.6 discipline)

No aliases. The `*Runtime` suffix was always a temporary marker (sub-DESIGNs 237.2 + 237.3 explicitly said "Stone 237.4 refines"). Rename everywhere; no compatibility shim.

---

## Substrate work breakdown

| # | File | Work |
|---|---|---|
| 1 | `src/runtime.rs` | Mint `ClauseAttempt` + `ClauseFailureReason`; rename + enrich the 2 RuntimeError variants; update Display arms; update construction sites (`runtime.rs:7172` + `7197` — the dispatch loop builds these errors) |
| 2 | `src/runtime.rs` | Dispatch loop must NOW RECORD the failure reason per skipped clause (arity / type / guard-false) — currently it just `continue`s; needs to accumulate `ClauseAttempt` entries |
| 3 | `src/runtime_error_edn.rs` | Rename + enrich the 2 EDN arms; clean tags; serialize `ClauseAttempt` + `ClauseFailureReason` |
| 4 | `src/check.rs` | If `CheckError::NoMatchingClauseAtCallSite` should also gain structured attempts — OPTIONAL; check-side may stay simpler (the runtime error is the teaching surface). DECISION: leave check-side variant as-is unless probe demands richness. |

**Dispatch-loop change (the load-bearing piece):** Stone 237.2/237.3's dispatch loop `continue`s on each non-matching clause. Stone 237.4 makes it ACCUMULATE a `ClauseAttempt` describing WHY it skipped — then if all clauses skip, the accumulated `Vec<ClauseAttempt>` becomes the error's `attempted_clauses`. This is the substantive change; the rename + EDN is mechanical.

---

## FM 2-bis probe — pre-stone authoring

**File:** `tests/probe_arc237_stone4_rich_errors.rs`

**Probe contracts (10):**

```rust
// Probe 1 — NoMatchingClause renamed (NoMatchingClauseRuntime grep returns 0 in src/)
#[test] fn probe_01_no_matching_clause_renamed_clean() { ... }

// Probe 2 — PostconditionFailed renamed (PostconditionFailedRuntime grep returns 0)
#[test] fn probe_02_postcondition_failed_renamed_clean() { ... }

// Probe 3 — NoMatchingClause EDN tag is #wat.kernel/NoMatchingClause
#[test] fn probe_03_no_matching_clause_edn_tag_clean() { ... }

// Probe 4 — PostconditionFailed EDN tag is #wat.kernel/PostconditionFailed
#[test] fn probe_04_postcondition_failed_edn_tag_clean() { ... }

// Probe 5 — NoMatchingClause attempt list shows ArityMismatch reason
#[test] fn probe_05_attempt_shows_arity_mismatch() { ... }

// Probe 6 — NoMatchingClause attempt list shows ArgTypeMismatch reason
#[test] fn probe_06_attempt_shows_type_mismatch() { ... }

// Probe 7 — NoMatchingClause attempt list shows GuardFalse reason
#[test] fn probe_07_attempt_shows_guard_false() { ... }

// Probe 8 — PostconditionFailed carries ensure_expr_snapshot + returned_value
#[test] fn probe_08_postcondition_carries_ensure_snapshot() { ... }

// Probe 9 — Stone 237.2 regression (defclause foundation still errors on no-match)
#[test] fn probe_09_stone_237_2_no_match_still_errors() { ... }

// Probe 10 — Stone 237.3 regression (guard-false fall-through still errors)
#[test] fn probe_10_stone_237_3_guard_false_still_errors() { ... }
```

10 contracts. Pre-stone: probes 1-8 FAIL (old names + flat structure); probes 9-10 PASS (behavior unchanged). Post-stone: 10/10 PASS.

**Note on probe approach:** EDN-tag + attempt-structure probes require invoking the error + inspecting its EDN serialization OR pattern-matching the RuntimeError variant. The probe will use `startup_from_source` + `eval_in_frozen` to trigger the error, then inspect the returned `RuntimeError` (via the error's EDN serialization or Display, whichever the existing arc 233.3 probe pattern uses). Mirror `tests/probe_stone_233_3_runtime_error_edn.rs` for the inspection technique.

---

## Trap-door audit

1. **Dispatch-loop accumulation.** The loop currently `continue`s; must now build `ClauseAttempt` per skip. Trap: the loop must distinguish WHY it skipped (arity vs type vs guard). The arity check + type check + guard eval are separate steps; each must record its specific reason. Probes 5/6/7 verify each reason fires.

2. **GuardFalse vs guard-error.** A clause skipped for `:guard false` records `GuardFalse`. A clause where `:guard` RAISES (runtime error) does NOT record an attempt — it propagates the error (per Stone 237.3 trap-door 2). Preserve this distinction.

3. **Rename cascade.** `NoMatchingClauseRuntime` + `PostconditionFailedRuntime` appear in: variant def, Display arms, EDN arms, variant_name, construction sites, AND any tests pattern-matching the names. Grep all; rename all; HARD CUT.

4. **EDN tag stability.** arc 233.3's EDN tags follow `#wat.kernel/<VariantName>`. The clean names produce `#wat.kernel/NoMatchingClause` + `#wat.kernel/PostconditionFailed`. Verify against the `tagged(...)` helper's tag-prefix convention.

5. **`ClauseFailureReason` EDN shape.** Sub-enum serialization — each variant (`ArityMismatch` / `ArgTypeMismatch` / `GuardFalse`) needs an EDN representation. Follow arc 233.3's enum-in-error serialization pattern.

6. **Check-side variant.** `CheckError::NoMatchingClauseAtCallSite` — decide whether it ALSO gets structured attempts. Lean: NO (the runtime error is the teaching surface; check-side catches at compile time with simpler info). Only enrich if a probe demands it.

7. **Stone 237.2 + 237.3 regression.** Those probes test that errors OCCUR (is_err), not exact variant names. The rename should NOT break them. Verify probes 9/10 + the full 237.2 (12) + 237.3 (14) suites stay green.

8. **dual-span for PostconditionFailed.** `body_span` (where body produced value) + `ensure_span` (where :ensure :fn declared). Both must be captured at construction time in the dispatch loop. The clause carries both AST nodes; extract spans from them.

---

## Tests (load-bearing for SCORE)

**Substrate probe (10 contracts).**

**Lib tests must stay GREEN:** `cargo test --release --lib -p wat` 827 PASS.

**Clippy** NOT a ceiling concern (arc 109 closure sweeps).

**Integration regression:**
- Stone 237.3 probe 14/14
- Stone 237.2 probe 12/12
- Stone 237.1 probe 14/14
- arc 233.3 probe (RuntimeError EDN) still green — CRITICAL since this stone touches runtime_error_edn.rs
- arc 234 / arc 236 probes green

---

## Calibration

| | Estimate |
|---|---|
| Predicted cascade rounds | 2-3 |
| Predicted runtime | **45-90 min Mode A** |
| STOP | **180 min** |
| New Value variants | 0 |
| New structs | 1 (ClauseAttempt) + 1 enum (ClauseFailureReason) |
| New RuntimeError variants | 0 (RENAME + enrich 2 existing) |
| New CheckError variants | 0 (unless probe demands check-side richness) |
| Test rot risk | LOW-MEDIUM (rename touches multiple sites; behavior unchanged) |

LIGHTER than Stones 237.2/237.3 — no new Value variant, no new dispatch mechanism. The dispatch-loop accumulation is the only substantive logic; the rest is rename + EDN + struct minting (mechanical per arc 233 precedent).

Stone 233.3 (Errors-as-EDN, 28 variants) is the closest precedent. Likely 30-50 min actual per pre-emption trend.

---

## Substrate dependencies (all GREEN)

- Stone 237.3 (ee5e892c) — PostconditionFailedRuntime variant + dispatch loop
- Stone 237.2 (bdd9eb6c) — NoMatchingClauseRuntime variant + dispatch loop
- arc 233.3 — EDN-serialization pattern (mirror for the clean tags)
- arc 138 — span coordinates

---

## Cross-references

### Within arc 237
- `DESIGN.md` umbrella — Stone 237.4 row
- `DESIGN-STONE-237.2.md` + `DESIGN-STONE-237.3.md` — the temporary variants this stone refines
- `SCORE-STONE-237.3.md` — most recent ship; runtime_error_edn.rs touched there too

### Substrate precedents
- `src/runtime.rs:2216` + `:2231` — the variants to rename + enrich
- `src/runtime.rs:7172` + `:7197` — construction sites in the dispatch loop
- `src/runtime_error_edn.rs:251` + `:267` — EDN arms to refine
- `tests/probe_stone_233_3_runtime_error_edn.rs` — EDN-inspection probe technique to mirror

### Doctrine
- arc 233 — errors as teaching values; ValueSnapshot + Provenance + EDN
- `feedback_no_known_defect_left_unfixed` — the flat-string attempt list is a known diagnostic gap; close it
- arc 234.6 HARD CUT — rename temporary names; no aliases

---

## Next moves (after sub-DESIGN nod)

1. Author `tests/probe_arc237_stone4_rich_errors.rs` — 10 contracts
2. Commit probe (BEFORE BRIEF)
3. Author `BRIEF-STONE-237.4.md` + `EXPECTATIONS-STONE-237.4.md`
4. Commit BRIEF + EXPECTATIONS
5. Spawn sonnet
6. On return: SCORE + commit + roll into Stone 237.5 (variadic + widest-contagion) per user direction "a then b"

---

*The dungeon's fourth chamber. The errors learn to teach — WHY each clause failed, not just THAT none matched. After 237.4, the diagnostic surface matches arc 233's doctrine.*
