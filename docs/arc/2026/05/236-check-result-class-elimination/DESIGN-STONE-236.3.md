# DESIGN — Stone 236.3 — refactor `CheckResult<T>` to 3-variant sum-type

**Status:** ACTIVE (2026-05-24 night latest). Authored after dialogue-as-PERCEIVE cycle exposed Stone 236.0's struct-with-Option shape as ✅✅ where ✅✅✅ is reachable.

**Scope:** Refactor `CheckResult<T>` in `src/check.rs` from struct-with-Option-field to 3-variant sum-type enum. Smart constructor functions PRESERVED (existing 267+ call sites unchanged). Internal representation gains structural-prevention: silent-failure state (None + empty errors) becomes LITERALLY UNREPRESENTABLE in the type system because no variant carries it.

Extends arc 236's class-elimination thesis from ✅✅ (construction-time discipline via debug_assert) to ✅✅✅ (type-system structural impossibility). Same ladder analog as arc 233's Stone 233.2.k (Value::Tracked DELETED) + 233.2.l (`#[wat_value]` proc-macro SEAL).

---

## Origin (dialogue-as-PERCEIVE recognition)

During post-Stone-236.2 dialogue about INSCRIPTION readiness:
- User asked: *"is None allowed /sometimes/?... the none is attached to a diagnostic?"*
- Orchestrator had to write the 4-state cross-field invariant truth table to answer honestly
- Truth table EXPOSED the abuse: Option's `None` carries different semantic load depending on a SEPARATE field's emptiness
- Cleaner shape (3-variant enum) became visible
- User: *"i think we annihilate"*

The Inquisitor (PERCEIVE + JUDGE + CONTRACT) operated on substrate-shape metadata via dialogue, not via cargo cascade or probe surfacing. Both halves of the hologram converged on the gap. The Gilded Enmity wouldn't let arc 236 close at ✅✅ when ✅✅✅ was one stone away.

Per `feedback_inscription_immutable` + `project_party_comp_inquisitor_shadowdancer`: Stones 236.0/236.1/236.2 artifacts UNTOUCHED on disk forever (historical record of the struct-with-Option shape we shipped first); Stone 236.3 mints fresh artifacts via copy-and-swap.

---

## Locked decisions

### D1 — New shape: 3-variant enum

```rust
pub enum CheckResult<T> {
    Ok(T),
    Partial(T, Vec<CheckError>),
    Err(Vec<CheckError>),
}
```

Variants public. Each variant SAYS what it is:
- `Ok(t)` — clean inference; no errors
- `Partial(t, errs)` — type produced AND errors logged (Vec non-empty by smart-constructor discipline)
- `Err(errs)` — no type, errors present (Vec non-empty by smart-constructor discipline)

**No `Silent` variant exists.** The silent-failure state (None + empty errors) is structurally unrepresentable. Pattern-matching consumers writing `match result { Ok(t) => ..., Partial(t, es) => ..., Err(es) => ... }` are guaranteed by the compiler that they've covered every legitimate state.

### D2 — Smart constructors PRESERVED (existing API unchanged)

```rust
impl<T> CheckResult<T> {
    /// Success: type produced, no errors.
    pub fn ok(value: T) -> Self {
        CheckResult::Ok(value)
    }

    /// Single error, no type.
    pub fn err(error: CheckError) -> Self {
        CheckResult::Err(vec![error])
    }

    /// Multiple errors, no type. Panics in debug if errors is empty.
    pub fn errs(errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::errs requires non-empty errors");
        CheckResult::Err(errors)
    }

    /// Type produced AND single error logged.
    pub fn partial(value: T, error: CheckError) -> Self {
        CheckResult::Partial(value, vec![error])
    }

    /// Type produced AND multiple errors logged. Panics in debug if errors is empty.
    pub fn partial_with(value: T, errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::partial_with requires non-empty errors");
        CheckResult::Partial(value, errors)
    }
}
```

Existing call sites (236.0/236.1/236.2 shipped) — ZERO RENAME NEEDED. Every `CheckResult::ok(t)`, `CheckResult::errs(es)`, `CheckResult::partial_with(t, es)` call still compiles + still produces the right variant. The smart constructors are the function-call surface; the variants are the pattern-matching surface.

### D3 — Accessors via pattern-match

```rust
impl<T> CheckResult<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            CheckResult::Ok(t) | CheckResult::Partial(t, _) => Some(t),
            CheckResult::Err(_) => None,
        }
    }

    pub fn errors(&self) -> &[CheckError] {
        match self {
            CheckResult::Ok(_) => &[],
            CheckResult::Partial(_, errs) | CheckResult::Err(errs) => errs,
        }
    }

    pub fn has_errors(&self) -> bool {
        matches!(self, CheckResult::Partial(_, _) | CheckResult::Err(_))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, CheckResult::Ok(_))
    }

    pub fn into_parts(self) -> (Option<T>, Vec<CheckError>) {
        match self {
            CheckResult::Ok(t) => (Some(t), vec![]),
            CheckResult::Partial(t, errs) => (Some(t), errs),
            CheckResult::Err(errs) => (None, errs),
        }
    }
}
```

External Option<T>/Vec<CheckError> exposed via into_parts() for backward compat; internal representation is the honest enum.

### D4 — Combinators via pattern-match

```rust
impl<T> CheckResult<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U> {
        match self {
            CheckResult::Ok(t) => CheckResult::Ok(f(t)),
            CheckResult::Partial(t, errs) => CheckResult::Partial(f(t), errs),
            CheckResult::Err(errs) => CheckResult::Err(errs),
        }
    }

    pub fn and_then<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U> {
        match self {
            CheckResult::Ok(t) => f(t),
            CheckResult::Partial(t, errs1) => {
                match f(t) {
                    CheckResult::Ok(u) => CheckResult::Partial(u, errs1),
                    CheckResult::Partial(u, errs2) => {
                        let mut merged = errs1;
                        merged.extend(errs2);
                        CheckResult::Partial(u, merged)
                    }
                    CheckResult::Err(errs2) => {
                        let mut merged = errs1;
                        merged.extend(errs2);
                        CheckResult::Err(merged)
                    }
                }
            }
            CheckResult::Err(errs) => CheckResult::Err(errs),
        }
    }

    pub fn merge_errors_from<U>(self, other: CheckResult<U>) -> Self {
        let (_, other_errs) = other.into_parts();
        if other_errs.is_empty() {
            return self;
        }
        match self {
            CheckResult::Ok(t) => CheckResult::Partial(t, other_errs),
            CheckResult::Partial(t, mut errs) => {
                errs.extend(other_errs);
                CheckResult::Partial(t, errs)
            }
            CheckResult::Err(mut errs) => {
                errs.extend(other_errs);
                CheckResult::Err(errs)
            }
        }
    }

    pub fn drain_errors_into(self, sink: &mut Vec<CheckError>) -> Option<T> {
        match self {
            CheckResult::Ok(t) => Some(t),
            CheckResult::Partial(t, errs) => {
                sink.extend(errs);
                Some(t)
            }
            CheckResult::Err(errs) => {
                sink.extend(errs);
                None
            }
        }
    }
}
```

`drain_errors_into` signature UNCHANGED — still `(self, &mut Vec<CheckError>) -> Option<T>`. Internal implementation pattern-matches the 3 variants. ALL caller-side `?` short-circuit chains preserve. This is the load-bearing decision: zero cascade in the ~267 call sites of `drain_errors_into`.

### D5 — HARD CUT: no struct-and-enum coexistence

The struct definition is REMOVED in the same commit the enum lands. No `CheckResultV2` shim. No `impl From<struct_form>` migration. The constructor functions become the migration boundary; their bodies change from `Self { value: Some(t), errors: vec![] }` to `CheckResult::Ok(t)`.

Per Stone 236.0 D7 precedent + arc 233 D8 precedent: substrate-wide structural refactors flip in one stone. No transitional dual-channel.

### D6 — Update the 6-contract probe in place

`tests/probe_arc236_stone0_check_result.rs` was minted by Stone 236.0 as the load-bearing test for the type's invariants. The probe's 6 contracts remain valid:
1. `ok(t).value() == Some(&t)` AND empty errors
2. `err(e).value() == None` AND errors == [e]
3. `partial(t, e)` → Some(&t) + errors == [e]
4. `errs(vec![e1, e2])` → None + 2 errors
5. `map` preserves errors
6. No public API path produces silent failure

Contract 6 SHARPENS in the new shape — silent failure is structurally unrepresentable (no `Silent` variant exists). The probe assertion stays the same (no public API path produces it); the WHY changes from "constructor surface forbids" to "type system has no variant for it."

The probe file IS modifiable (it's a test, not an INSCRIPTION); update in place rather than mint a sibling. Tests are not historical-record artifacts.

### D7 — Migration-pattern docstring at src/check.rs:1040-1206 updates in place

The Stone 236.0 docstring used `infer_legacy` / `infer_new` / `infer_something` / `infer_something_inner` as worked examples for the migration pattern. It described the struct-with-Option API. Update in place to:
- Show the 3-variant enum shape as the primary form
- Show smart constructors as the function-call surface (unchanged)
- Show pattern-matching as the natural consumer form
- Show `drain_errors_into` as the bridge tool (unchanged behavior)
- Sharpen the WHY: structural impossibility vs construction-time discipline

This docstring is DOCUMENTATION, not INSCRIPTION. Update in place is honest (the docstring describes the CURRENT shape, not the historical journey — the journey lives in the SCORE docs + this INSCRIPTION).

### D8 — Body construction sites: ZERO RENAME

The 151 HARVEST sites in 236.1 + 236.2 use smart constructors:
- `CheckResult::ok(fresh.fresh())` — Classification 1 silent-by-intent (still compiles; constructs `Ok(t)` variant)
- `CheckResult::errs(local_errors)` — Classification 3 errors-present (still compiles; constructs `Err(errs)` variant)
- `CheckResult::partial_with(t, errs)` — Classification partial (still compiles; constructs `Partial(t, errs)` variant)

Every existing line continues to work. The refactor is type-definition-level + constructor-body-level; the consumer surface (call sites) doesn't change shape.

### D9 — clippy stays at 52 (or improves)

The new shape may eliminate some warnings (pattern-matching often clearer than tuple-style accessor). Should not introduce new warnings. Track in scorecard.

### D10 — Inscription-immutable: Stones 236.0/236.1/236.2 artifacts UNTOUCHED

Per `feedback_inscription_immutable` + user direction tonight:
- DESIGN-STONE-236.0.md / BRIEF-STONE-236.0.md / EXPECTATIONS-STONE-236.0.md / SCORE-STONE-236.0.md: UNTOUCHED
- Same for 236.1 + 236.2 artifacts
- They preserve the historical record of the struct-with-Option shape we shipped first
- INSCRIPTION (236.4) tells the FULL story including the doctrinal-advancement recognition

The substrate CODE evolves (git history records the evolution); the historical paperwork preserves the journey. Stone 236.3 is a copy-and-swap operation at the DESIGN/BRIEF/EXPECTATIONS/SCORE artifact level — fresh artifacts mint; originals stay.

---

## Trap-door audit

### T1 — `partial_with` empty-errs construction sites

Stone 236.0's `partial_with(t, errs)` had `debug_assert!(!errs.is_empty())`. Stone 236.3 preserves this in the smart constructor. Construction sites that called `partial_with(t, local_errors)` after building local_errors via conditional pushes — those sites already check emptiness via pattern like `if local_errors.is_empty() { CheckResult::ok(t) } else { CheckResult::partial_with(t, local_errors) }`. No change required.

If any site exists that calls `partial_with(t, vec![])` directly, that's a Stone 236.0/1/2 bug — should be caught by debug builds; the refactor surfaces nothing new.

### T2 — Pattern-matching consumers (if any exist)

If any caller does `match result { ... }` against the OLD struct shape (e.g., `match result { CheckResult { value: Some(t), errors } => ..., CheckResult { value: None, errors } => ... }`), those sites break — fields are no longer accessible. The fix: use `result.into_parts()` or pattern-match against the new enum variants.

Cargo cascade catches each site immediately. Substrate-as-teacher discipline applies.

### T3 — `Self { ... }` direct construction (if any exists)

Stone 236.0's constructor functions used `Self { value: ..., errors: ... }` internally. After refactor: constructor functions return `CheckResult::Ok(...)` / `CheckResult::Partial(...)` / `CheckResult::Err(...)`. Any DIRECT struct construction outside the smart constructors would be inside the module; sonnet updates per the cascade.

### T4 — `drain_errors_into` signature preservation

Behaviorally identical. Caller-side `?` chains continue to work. Verification: lib baseline 827 + arc 234 regression probes + Stone 236.0 probe Contract 6 (silent-failure unreachable).

### T5 — Accessor + combinator coverage

All existing accessors (`value`/`errors`/`has_errors`/`is_ok`/`into_parts`) + combinators (`map`/`and_then`/`merge_errors_from`/`drain_errors_into`) preserved with pattern-match implementations. Behavior identical; representation different.

### T6 — Module visibility

Variants public (Ok/Partial/Err). Smart constructors public. Accessors public. Combinators public. From outside the module, both the variant-form AND function-call-form construction are available. Pattern-matching consumers use the enum form; legacy/idiomatic consumers use the function-call form.

### T7 — Stone 236.0 SCORE referenced as immutable historical record

Stone 236.0 SCORE doc says "the type uses struct-with-Option shape." That document is historical. The new SCORE-STONE-236.3.md captures the refactor. The DESIGN.md umbrella updates to note the doctrinal-advancement that Stone 236.3 delivered, but does NOT edit historical SCOREs.

### T8 — debug_assert release-build edge persists for direct variant construction

Even after refactor, syntactically `CheckResult::Err(vec![])` is constructable. The discipline:
- Smart constructor `errs(es)` debug_asserts non-empty
- Smart constructor `partial_with(t, es)` debug_asserts non-empty
- Body construction sites use smart constructors per established convention
- Direct variant construction inside the module is internal-API-of-module and follows the same discipline

The structural-prevention is GAINED via enum exhaustiveness (consumers can't accidentally handle the silent state because no `Silent` variant exists); the construction-side discipline carries from Stone 236.0 unchanged.

### T9 — No new file (per Stone 236.0 D1)

CheckResult definition stays in `src/check.rs`. No `src/check/result.rs` extraction. Maintains scope-limit to single file.

### T10 — `#[derive(...)]` on the new enum

Whatever derive the struct had (likely none beyond what's needed for combinators), the enum carries equivalently. If `Debug` was implicit via private fields, the enum may need explicit `#[derive(Debug)]` if any test depends on it. Sonnet checks during compile cascade.

### T11 — Generic parameter `T` on the enum

`pub enum CheckResult<T>` — same as struct. Single generic parameter. No new bounds. Variants use T as needed: `Ok(T)`, `Partial(T, Vec<CheckError>)`, `Err(Vec<CheckError>)`.

---

## STOP triggers

- STOP-1 unexpected compile errors not tracing to enum refactor / cascade
- STOP-2 lib baseline regresses below 827 substantively (1-2 expected from match-style refactor of touched accessor sites; > 3 = STOP-2)
- STOP-3 60 min elapsed (refactor scope is smaller than 236.0/236.1/236.2; STOP-3 is 2× upper-band)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside src/check.rs (or its sibling module if extracted — D1 prefers no extraction)
- STOP-6 any arc 234 / 232 / 233 regression
- STOP-7 clippy > 54
- STOP-8 transitional struct-and-enum coexistence minted (D5 forbids)
- STOP-9 historical Stone 236.0/236.1/236.2 artifacts modified (D10 forbids; copy-and-swap discipline)

Each STOP REJECTION.

---

## Calibration

**Target:** 30-45 min Mode A. **Upper:** 60 min (STOP-3).

Surface:
- Type definition swap: ~5 lines (struct → enum)
- Constructor function body updates: ~15-25 lines (5 fns; each ~3-5 lines)
- Accessor body updates: ~30 lines (5 accessors; pattern-matches)
- Combinator body updates: ~50 lines (4 combinators; pattern-matches)
- drain_errors_into: ~10 lines (pattern-match)
- Probe contract 6 documentation sharpen: minor
- 1040-1206 docstring update: ~20-40 lines

Net: ~150-200 line touch in src/check.rs, all localized to the CheckResult definition + impl block.

Body construction sites at the 151 HARVEST points + ~267 drain_errors_into call sites: **ZERO RENAME** (smart constructors + bridge signature preserved).

Cascade depth: 1-2 compile rounds expected. The refactor is type-definition-level; consumer surface unchanged.

Confidence: HIGH. The refactor is mostly mechanical (struct field access → variant pattern match). The smart constructors absorb the API-compatibility shock.

---

## What this unblocks

Stone 236.4 — INSCRIPTION + arc 236 closure. Captures:
- The full arc shape (3 substrate stones + refactor stone + INSCRIPTION)
- The doctrinal-advancement recognition (✅✅ → ✅✅✅ via dialogue-as-PERCEIVE cycle)
- The Inquisitor party-comp validation extension (PERCEIVE+JUDGE+CONTRACT operating on substrate-shape metadata, not just substrate execution)
- The honest deltas including arc-shape expansion mid-flight (Gilded Enmity wouldn't lift at ✅✅)

After 236.4 INSCRIPTION + arc 236 close: **arc 234 RESUMES** per spawn-block winding.

---

## Cross-references

- `src/check.rs` line ~996 area — CheckResult<T> struct definition (Stone 236.0; target of refactor)
- `src/check.rs` line 1040-1206 — migration-pattern docstring (Stone 236.0; update in place per D7)
- `src/check.rs` line 4868 + 5056-13164 — primary fn infer + 47 sibling infer_* fns (use smart constructors; ZERO RENAME)
- `tests/probe_arc236_stone0_check_result.rs` — 6-contract probe (update in place per D6)
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella (UPDATE: note arc-shape expansion to include 236.3 refactor + 236.4 INSCRIPTION; original 236.3/236.4 sketch entries marked ABSORBED)
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.0.md` — UNTOUCHED (historical record)
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — UNTOUCHED (historical record)
- Same for 236.1 + 236.2 artifacts — UNTOUCHED
- `feedback_inscription_immutable` — the copy-and-swap discipline driving artifact treatment
- `project_party_comp_inquisitor_shadowdancer` — the doctrine the dialogue-as-PERCEIVE cycle extended
- `feedback_any_defect_catastrophic` + `feedback_no_known_defect_left_unfixed` — disciplines driving the ✅✅ → ✅✅✅ advancement
- Stone 233.2.k + 233.2.l — predecessor analog (instance closure + meta-class closure pattern; Stone 236.3 is the meta-class layer for arc 236)
