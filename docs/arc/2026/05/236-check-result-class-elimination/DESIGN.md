# Arc 236 — check.rs error-propagation class-elimination

**Status:** SHIPPED 2026-05-25. Arc 236 CLOSED. See `INSCRIPTION.md` for the full closure record. Arc 234 RESUMES per spawn-block winding (`feedback_spawn_block_winding`).

**Final arc-shape:** 4 substrate stones (236.0/1/2/3) + 1 INSCRIPTION stone (236.4) = 5 stones total. Original 6-stone sketch compressed (236.3 audit + 236.4 verification ABSORBED by Stone 236.2's HARVEST methodology + 12-row scorecard) AND extended mid-flight (new 236.3 sum-type refactor recognized via dialogue-as-PERCEIVE cycle; elevates from ✅✅ construction-time discipline to ✅✅✅ type-system structural impossibility). Per `feedback_inscription_immutable`: compression is honest (work delivered via different stone-shape than predicted); extension is honest (doctrinal-advancement recognition emerged mid-arc; the discipline says ratchet when reachable).

**Origin:** Stone 234.3c.fix-narrow-fallthrough surfaced (and documented in commit `aa55505b`) the substrate-architecture failure mode: `check.rs::infer(...) -> Option<TypeExpr>` + `errors: &mut Vec<CheckError>` side-channel allows silent error-loss. `return None` without `errors.push(...)` produces no diagnostic.

We hit it twice today. Per failure-engineering doctrine: eliminate the class.

---

## The failure class

```rust
fn infer(...) -> Option<TypeExpr>;     // ← inference result
                                       //   Some(ty) = success
                                       //   None     = ??? (no type or error?)
errors: &mut Vec<CheckError>           // ← side-channel for diagnostics
```

Three possible states per `infer` call:
1. **Success:** `return Some(ty)`; no errors pushed. → Honest.
2. **Partial success:** `return Some(ty_or_fresh)`; `errors.push(...)`. → Honest (error logged, inference continues).
3. **Error:** `return None`; `errors.push(...)`. → Honest (no type, error logged).
4. **SILENT FAILURE:** `return None`; **no** errors push. → 🐛 LOSS. The substrate has no diagnostic; user sees confusing downstream behavior or nothing at all.

State #4 is what arc 236 makes structurally impossible.

---

## The class-elimination strategy

Replace the dual-channel pattern with a single newtype that carries BOTH the inference result AND the accumulated errors, with constructors that prevent state #4:

```rust
pub struct CheckResult<T> {
    value: Option<T>,
    errors: Vec<CheckError>,
}

impl<T> CheckResult<T> {
    /// Type produced; no errors.
    pub fn ok(value: T) -> Self { ... }

    /// Type produced AND error(s) logged. Inference continues with the
    /// partial type; downstream sees both. The "partial success" pattern.
    pub fn partial(value: T, errs: Vec<CheckError>) -> Self {
        debug_assert!(!errs.is_empty(), "partial() requires at least one error");
        ...
    }

    /// No type produced. At least one error MUST be present —
    /// the silent-failure case is structurally unreachable.
    pub fn err(errs: Vec<CheckError>) -> Self {
        debug_assert!(!errs.is_empty(), "err() requires at least one error");
        ...
    }

    pub fn value(&self) -> Option<&T>;
    pub fn errors(&self) -> &[CheckError];
    pub fn into_parts(self) -> (Option<T>, Vec<CheckError>);

    // Combinators
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U>;
    pub fn and_then<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U>;
    pub fn merge_errors(self, other_errs: Vec<CheckError>) -> Self;
    pub fn or_propagate_errors(self, sink: &mut Vec<CheckError>) -> Option<T>;
}
```

The CONSTRUCTORS enforce the invariant:
- `ok(value)` — type present, no errors
- `partial(value, errs)` — type present, errors present (errs non-empty enforced)
- `err(errs)` — type absent, errors present (errs non-empty enforced)
- **NO `none_no_error()` constructor exists. Period.**

Authors of `infer`-like functions cannot accidentally produce state #4 because the type system literally has no constructor for it.

---

## Migration scope (estimated; sonnet investigates per stone)

The pattern lives throughout `check.rs` (~28,000 lines). Migration likely touches:
- `fn infer(...) -> Option<TypeExpr>` — primary inference entry; ~100s of call sites
- Sibling functions (`infer_list`, `infer_*` helpers) — return types flip uniformly
- Every call site that does `let ty = infer(arg, ...); errors.push(...);` — restructures to `let result = infer(arg, ...); errors.extend(result.errors());`
- Every `return None` site — audit for "honest None" vs "silent failure" and pick `ok(fresh)` / `partial` / `err`

The substrate-as-teacher cascade is where the real value emerges: once the type system forces the choice, all latent silent-failure sites surface. Could be many or few; we find out.

---

## Stone sketch — original + actuals

**Original sketch (drafted 2026-05-24 early; preserved as history):**

- 236.0 Mint `CheckResult<T>` newtype; constructors + combinators + tests. NO migration yet.
- 236.1 Migrate primary `fn infer(...)` signature; substrate-as-teacher cascade.
- 236.2 Migrate sibling inference helpers; cascade continues.
- 236.3 Audit & fix surfaced silent-failure sites — failure-class HARVEST.
- 236.4 Lib baseline + regression guards green.
- 236.5 INSCRIPTION + close.

> May expand to 6-8 stones depending on cascade depth.

**Actuals (as of 2026-05-24 night):**

- **236.0** SHIPPED (`63f8ca2a`) — CheckResult<T> struct-with-Option mint (11/11; ~25 min)
- **236.1** SHIPPED (`f06549ad`) — primary fn infer flip (11/11; HARVEST 2/0/1; ~25 min)
- **236.2** SHIPPED (`d8aa66d0`) — sibling infer_* flip + HARVEST methodology (12/12; HARVEST 37/0/111; ~57 min); ABSORBED original 236.3 (audit work via HARVEST methodology; 0 Classification 2 sites yielded) + original 236.4 (verification work via 12-row scorecard; 827 lib + clippy 52 + all regression probes green)
- **236.3** (renumbered; ACTIVE) — `CheckResult<T>` sum-type refactor: struct-with-Option → 3-variant enum `Ok(T) | Partial(T, Vec<CheckError>) | Err(Vec<CheckError>)`. Extends class-elimination ✅✅ (construction-time discipline) → ✅✅✅ (type-system structural impossibility). Silent-failure state literally unrepresentable because no `Silent` variant exists. Recognized via dialogue-as-PERCEIVE cycle post-236.2.
- **236.4** (renumbered; pending Stone 236.3 ship) — INSCRIPTION + arc closure.

Arc shipped its substrate work in 3 stones (vs original 6-stone sketch); extended mid-flight by 1 stone for the doctrinal-advancement refactor (Stone 236.3); closes with INSCRIPTION (Stone 236.4). Total: 4 substrate stones + 1 closure stone.

---

## What this protects forward

After arc 236 closes:
- Authors writing new `infer_*` helpers literally cannot produce silent-error-loss. Type system enforces.
- Future check.rs work (arc 232.1 defprotocol check-time, per-class TypeDef registration, etc.) inherits the discipline by default.
- The honest-error principle that 234.3b.fix + 234.3c.fix-narrow-fallthrough were ad-hoc forms of becomes the substrate norm.

---

## Sequencing — arc 234 RESUMES after 236

Per spawn-block winding + user direction: arc 236 closes; arc 234 resumes; arc 234 closes; arc 235 (PROPOSED) opens.

Arc 234 remaining work (per `docs/arc/2026/05/234-wat-record-hologram/PAUSE-CONTEXT.md`):
- 234.4.match (small; let → match parity for hash-destructure)
- 234.6 (migration sweep — possibly its own arc 238 separate from 234)
- 234.7 INSCRIPTION

---

## STOP triggers (arc-level)

- **holon-rs touched** — substrate is frozen
- **arc 234 probes regress** — the failure-class elimination must not introduce regressions in the live record substrate
- **lib baseline < 827** — strict-no-regression
- **scope creep beyond check.rs error-propagation** — runtime.rs, parser.rs, etc. are out of scope
- **silent-failure sites discovered + skipped** — when 236.3 audits, every site gets honest handling; no "fix later" deferrals (per `feedback_no_known_defect_left_unfixed`)

---

## Calibration (arc-level estimate)

**Target:** 4-8 stones; multi-day work.
**Confidence:** medium — cascade depth unknown until stone 1 ships and substrate-as-teacher emits compile errors.

Single-stone calibrations TBD per stone.

---

## Cross-references

- `src/check.rs` — the file under treatment
- `src/check.rs` line ~5900 area — the recent fall-through site that surfaced the failure mode (Stone 234.3c.fix-narrow-fallthrough)
- `feedback_any_defect_catastrophic.md` — discipline driving this arc
- `project_failure_engineering.md` — pattern driving this arc
- `feedback_refuse_easy_solutions.md` — why not "doctrine + audit" (option 1) instead
- `feedback_no_known_defect_left_unfixed.md` — the deferral discipline that catches "future cleanup" rationalization
- `docs/arc/2026/05/234-wat-record-hologram/PAUSE-CONTEXT.md` — what arc 234 left behind
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — arc 233 precedent (errors-as-EDN; same family of failure-engineering work)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
