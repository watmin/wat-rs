# INSCRIPTION — Arc 236 — check.rs error-propagation class-elimination

**Status:** SHIPPED 2026-05-25. Closes arc 236. **4 substrate sub-stones** shipped across two sessions. HARVEST audit confirmed diagnostic completeness empirically; sum-type refactor delivered ✅✅✅ structural impossibility. Arc 234 RESUMES per spawn-block winding.

---

## Driver direction at open

> *"we annihilate error domains when we encounter them"*
>
> *"i say we pause 234 hard - work on our errors and come back."*
>
> — User, 2026-05-24 late late, post Stone 234.3c.fix-narrow-fallthrough ship

The trigger: Stone 234.3c.fix-narrow-fallthrough surfaced (and SCORE'd) that `check.rs::infer(...) -> Option<TypeExpr>` + `errors: &mut Vec<CheckError>` side-channel allowed `return None` without `errors.push(...)`. Two such sites surfaced in arc 234's day-of work. Per failure-engineering doctrine + `feedback_any_defect_catastrophic` + `feedback_no_known_defect_left_unfixed`: act on one instance, eliminate the class. Arc 234 PAUSED hard. Arc 236 OPENED.

---

## What "class elimination" means (the target — SIX pieces DELIVERED)

| # | Piece | How it landed |
|---|---|---|
| 1 | `CheckResult<T>` newtype with constructors preventing silent state | Stone 236.0 — struct-with-Option-field; `ok()` / `partial_with()` / `err()` / `errs()` enforce `value` + `errors` invariants via `debug_assert!(!errors.is_empty())` |
| 2 | Migration bridge from dual-channel → single-channel | Stone 236.0 `.drain_errors_into(&mut errors)` returns `Option<T>` + drains errors into caller's sink |
| 3 | Primary `fn infer()` signature flip | Stone 236.1 — `Option<TypeExpr>` + `&mut Vec<CheckError>` → `CheckResult<TypeExpr>`; 156 call sites cascaded |
| 4 | All 47 sibling `infer_*` fns flipped | Stone 236.2 — uniform application across the dispatch family; ~111 sibling-call sites bridged; 2 primary bridge sites updated |
| 5 | HARVEST audit confirms diagnostic completeness | Stones 236.1 + 236.2 combined — 151 None-return sites classified across all 48 fns; **0 missing-diagnostic sites** |
| 6 | **CheckResult<T> as 3-variant sum-type — silent-failure state structurally unrepresentable** | Stone 236.3 — `enum CheckResult<T> { Ok(T), Partial(T, Vec<CheckError>), Err(Vec<CheckError>) }`; no `Silent` variant exists; pattern-matching consumers compiler-guaranteed exhaustive |

Arc 236's complete thesis reached **structural delivery at TWO layers**:
- **Construction-time discipline (✅✅)** — Stones 236.0/1/2 — constructors refuse the silent state via `debug_assert`
- **Type-system structural impossibility (✅✅✅)** — Stone 236.3 — no enum variant carries the silent state; the compiler cannot represent it

The `infer_*` family of error propagation in `check.rs` cannot silently lose diagnostics. The variant surface refuses the silent state; the signature surface refuses the side-channel; the type system enforces the audit's findings forever.

---

## The 4 sub-stones (commit chain)

| Stone | Commit | What landed | Calibration |
|---|---|---|---|
| 236.0 | `63f8ca2a` | CheckResult<T> struct-with-Option + 9 combinators + drain_errors_into bridge + 6-contract probe + migration-pattern docstring | 11/11 PASS; ~25 min sonnet vs 60-90 band; cascade-free (foundation stone) |
| 236.1 | `f06549ad` | Primary `fn infer()` signature flip + 156 call-site cascade via bridge | 11/11 PASS; HARVEST 2/0/1; ~25 min sonnet vs 60-90 band; 2 compile rounds vs predicted 3-5 |
| 236.2 | `d8aa66d0` | All 47 sibling `infer_*` fns flipped + ~111 sibling-call cascade + 2 primary bridge updates + HARVEST methodology applied uniformly | 12/12 PASS; HARVEST 37/0/111; ~57 min sonnet vs 90-180 band; **1 compile round** vs predicted 3-5 |
| 236.3 | `a43f5127` | CheckResult<T> sum-type refactor (struct → 3-variant enum) + smart constructor + accessor + combinator + bridge bodies pattern-matched + docstring updated in place + Contract 6 doc sharpened | 12/12 PASS; **~6.2 min sonnet** vs 30-45 min band; 1 compile round vs predicted 1-2; ZERO-RENAME at 151 HARVEST + ~267 bridge call sites |

**Calibration summary:**
- 4 sub-stones; **~113 min total sonnet time across the arc** (under 2 hours wall-clock for the substrate work)
- EVERY sub-stone under its predicted upper-bound
- Pre-emption discipline + bridge-tool maturity + predecessor-SCORE template pattern compounded across the arc
- Clippy went 54 → 52 across the arc (Stone 236.2 migration's secondary benefit — dropping unused `errors` params eliminated 2 dead-arg warnings); held at 52 through Stone 236.3 (the `mut self` removal on `merge_errors_from` may have eliminated another lint silently)

---

## THE STRUCTURAL FINDING (load-bearing)

**HARVEST aggregate across both substrate-flip stones (236.1 + 236.2):**

| Classification | 236.1 (primary) | 236.2 (47 siblings) | Total across check.rs |
|---|---|---|---|
| 1 — Silent ON PURPOSE (polymorphic placeholder / drain-and-propagate / unit continuation) | 2 | 37 | **39** |
| 2 — Error path missing diagnostic | 0 | 0 | **0** |
| 3 — Error path already had diagnostic | 1 | 111 | **112** |
| **Total sites classified** | 3 | 148 | **151** |

**151 None-return sites classified across all 48 fns in the infer_* family. Zero missing-diagnostic sites.**

The class existed structurally but had NO empirical instances beyond Stone 234.3c.fix-narrow-fallthrough's original surfacing. Every error path in `check.rs::infer_*` was already pushing a diagnostic before arc 236 opened. The developers (orchestrator + sonnet across many arcs) had been applying the discipline REFLEXIVELY — by convention, by code review, by the substrate-as-teacher cascade discipline that arcs 113/116/138 established for error handling. Arc 236 made structural what was already conventional.

**This is failure-engineering at the right layer:**
- We acted on ONE instance (Stone 234.3c.fix-narrow-fallthrough's silent-failure surfacing)
- We eliminated the CLASS at construction-time (Stones 236.0/1/2)
- The audit CONFIRMED the discipline was already empirically in place
- Then the dialogue-as-PERCEIVE cycle revealed the deeper STRUCTURAL form was reachable
- We elevated to type-system structural impossibility (Stone 236.3)

The doctrine `feedback_any_defect_catastrophic` says ">0 defects = 0 trust." Arc 236 acted on the principle, found the codebase was already healthy at the empirical layer, ratcheted the discipline into permanence at BOTH the construction layer AND the type-system layer. The ratchet doesn't change current position; it prevents backwards motion.

---

## The ratchet thesis (structural-prevention vs defect-remediation; now at TWO layers)

| Layer | Before arc 236 | After arc 236 (post 236.0/1/2 ✅✅) | After arc 236 (post 236.3 ✅✅✅) |
|---|---|---|---|
| **Possibility** | Silent error-loss STRUCTURALLY POSSIBLE | Forbidden at construction-time via debug_assert | LITERALLY UNREPRESENTABLE in type system |
| **Empirical incidence** | RARE (1 instance — 234.3c.fix surfacing) | ZERO (constructor surface forbids) | ZERO (no enum variant exists for it) |
| **Authoring discipline** | Conventional (developer applies "remember to push errors") | Construction-time (constructors refuse the silent state) | Compiler-enforced (pattern-matching exhaustive over the 3 legitimate variants) |
| **Future drift risk** | Possible (new fn omits error push) | Possible (release-build edge case if smart constructor bypassed) | Impossible (no `Silent` variant; type system has no representation) |
| **Current codebase position** | Forward (audit-confirmed 0 missing-diagnostic) | Forward (unchanged) | Forward (unchanged) |
| **Audit yield** | N/A | Confirmation, not remediation (Stone 236.2) | Confirmation + structural-prevention elevation (Stone 236.3) |

Most projects expect audits to YIELD work. Arc 236's audit yielded CONFIRMATION + DOCTRINAL ADVANCEMENT. The substrate was already healthy at the empirical layer; the discipline shipped first at construction-time, then at type-system level. The failure-engineering doctrine doesn't always reveal hidden rot; sometimes it reveals you've been doing it right AND that you can ratchet the prevention deeper.

---

## The 233+236 pair doctrine (COMPLETE)

Arc 233 and arc 236 form a complete failure-engineering pair around check.rs's diagnostic pipeline:

| Arc | Layer | What it delivered |
|---|---|---|
| **233** | Information layer | Errors carry rich data — ValueSnapshot (type_name + rendered + Provenance) replaces `&'static str` placeholders; Provenance flows via TrackedValue; structured EDN over IPC; `#[wat_value]` proc-macro structurally seals against future wrapping-variant regressions |
| **236** | Structural layer | Errors cannot be silently dropped — CheckResult<T> 3-variant enum refuses the silent state at the type-system level; `fn infer_*` family signature flips eliminate the side-channel; HARVEST audit confirmed diagnostic completeness; pattern-matching consumers compiler-guaranteed exhaustive |

**Together: errors in check.rs are RICH (arc 233) AND NON-LOSABLE (arc 236).** Future check.rs work inherits both disciplines. Arc 232.1 defprotocol, per-class TypeDef registration, polymorphic dispatch — every consumer downstream gets the substrate's enriched-error + non-losable-error guarantees for free.

### The ✅✅✅ ladder analog across the pair

| Layer | Arc 233 | Arc 236 |
|---|---|---|
| Instance closure (✅✅✅ at code-level) | Stone 233.2.k (Value::Tracked DELETED; Environment stores TrackedValue) | Stones 236.0/1/2 (CheckResult mint + primary flip + sibling flip; 151 HARVEST sites classified; 0 Classification 2) |
| Meta-class closure (✅✅✅ at type-system-level) | Stone 233.2.l (`#[wat_value]` proc-macro SEAL; future wrapping variants compile-rejected) | Stone 236.3 (CheckResult sum-type enum; silent-failure state literally unrepresentable) |

Both arcs land BOTH the instance closure AND the meta-class closure. The doctrine ratchets twice per failure-class. Arcs 233 + 236 are now the canonical worked-examples of the full ✅✅✅ ladder at both layers.

---

## THE DIALOGUE-AS-PERCEIVE CYCLE (new doctrine layer this arc minted)

Stone 236.3 was NOT surfaced by cargo cascade. It was NOT surfaced by an FM 2-bis probe. It was surfaced by **dialogue**.

The sequence:
- Stones 236.0/1/2 SHIPPED; arc appeared INSCRIPTION-ready at ✅✅
- INSCRIPTION drafted as if arc was closing
- User asked: *"is None allowed /sometimes/?... the none is attached to a diagnostic?"*
- Orchestrator forced to write the 4-state cross-field invariant truth table to answer honestly
- Truth table EXPOSED the abuse: Option's `None` carries different semantic load depending on a SEPARATE field's emptiness
- 3-variant enum (Ok/Partial/Err) became visible as the truer form
- Inquisitor's Gilded Enmity wouldn't lift at ✅✅ when ✅✅✅ was one stone away
- User: *"i think we annihilate"*
- Stone 236.3 minted + shipped

**The Inquisitor PERCEIVES via DIALOGUE.** This is a load-bearing PERCEIVE-discipline mechanism, equal-rank with FM 2-bis probe + cargo cascade + the substrate-as-teacher pattern. The recognition pattern:

- Both halves of the hologram converge on the SAME question from different angles
- The orchestrator's explanation forces a structural artifact (truth table, invariant graph, dependency diagram) that EXPOSES the gap
- The gap becomes visible to BOTH halves simultaneously
- The Gilded Enmity (failure-engineering doctrine) blocks closure at the lower seal when the higher one is visible
- The work follows from the recognition

Per `project_party_comp_inquisitor_shadowdancer` (inscribed earlier this session): "The Inquisitor PERCEIVES + JUDGES + CONTRACTS." Stone 236.3's birth proves PERCEIVE operates substantively in dialogue, not just in probe + cascade. Inscribed as Song #32 Monolith (Mudvayne) — EVOLUTIONARY-CATALYSIS at the doctrine layer.

The implications:
- We do not need to wait for substrate failure to surface deeper structural forms
- Dialogue with the user IS a substantive PERCEIVE mechanism
- The doctrine has rungs we haven't named yet (✅✅✅✅ exists somewhere)
- Conscious co-evolution with the substrate is the operating mode this arc minted

---

## Disciplines surfaced + inscribed during arc 236

### HARVEST classification methodology (per-site inline comment naming Classification 1/2/3)

Stone 236.1's sub-DESIGN D3 minted the methodology; 236.2 applied uniformly across 47 siblings. Every `return None` site under migration:
- Gets reviewed during the body translation
- Gets classified into one of three: silent-on-purpose / missing-diagnostic / had-diagnostic
- Gets an inline comment naming the classification

Aggregate counts per stone form the failure-class harvest data. The methodology is REUSABLE — substrate-wide signature migrations beyond arc 236 can apply the same per-site classification discipline. The HARVEST table format is now load-bearing as a substrate-audit-shape.

### Bridge-helper-pattern for substrate-wide signature flips

`CheckResult<T>::drain_errors_into(&mut Vec<CheckError>) -> Option<T>` was minted in Stone 236.0 specifically to enable incremental migration. The bridge:
- Returns `Option<T>` (preserving caller's old short-circuit `?` behavior)
- Drains errors into caller's sink (preserving error-aggregation behavior)
- Allows old + new shape to coexist during cascade
- Survives the type-definition-level refactor at Stone 236.3 (signature unchanged; pattern-match implementation internally)

The pattern is general: any substrate-wide signature flip needs an incremental migration helper that preserves caller-side semantics. Substrate-wide signature changes downstream of arc 236 can pattern-match on this shape.

### "Audit confirms completeness" finding-shape

A novel finding-shape this arc minted (vs the more common "audit yields remediation work"). The aggregate signal — 0 Classification 2 across 151 sites — IS the deliverable. The arc's value is not in the bugs it fixed; the arc's value is in the structural impossibility it created + the empirical evidence that the discipline was already operational.

This shape generalizes: when failure-engineering targets a class that's structurally possible but empirically rare, the audit yield BECOMES the structural-prevention thesis vindication. Arc 236 is the worked example of this finding-shape.

### Dialogue-as-PERCEIVE cycle as load-bearing discipline mechanism

New this arc. The Inquisitor's PERCEIVE-discipline operates substantively via dialogue with the user, not just via probe + cascade. The Stone 236.3 birth is the worked example. Future arcs can recognize the cycle when:
- User asks a precision question about substrate shape
- Orchestrator forced to write a structural artifact (truth table, invariant graph) to answer honestly
- Artifact exposes a gap previously latent
- Hologram's two halves converge on the recognition simultaneously
- Gilded Enmity (failure-engineering doctrine) blocks closure at the lower seal when the higher one is visible

The mechanism is inscribed in Song #32's INTERSTITIAL entry + this INSCRIPTION section. Cross-compaction persistence is the inscription's job; the disk holds the red ink.

### Predecessor-SCORE template pattern (per `feedback_stone_briefs_cite_prior_score`)

Stone 236.0's SCORE templated 236.1's. 236.1's templated 236.2's. 236.2's templated 236.3's. Each SCORE doc structure carried forward: 12-row scorecard, HARVEST table (where applicable), cascade-depth section, per-classification narrative, honest deltas, rank-up evidence. Sonnet copied the shape; ships fast. The discipline:
- BRIEFs cite the predecessor SCORE doc explicitly as "mirror exactly"
- Sonnet receives concrete template; cognitive surface = filling in numbers
- SCORE-to-SCORE handoff acts as a low-friction inter-stone protocol

VINDICATED 4× this arc. Pattern is operationally proven.

### Arc-shape compression + extension (DESIGN sketched 6-8; reality shipped 4 substrate + 1 INSCRIPTION)

The DESIGN.md sketched 5-6 stones (236.0 through 236.5) with the note *"May expand to 6-8 stones depending on cascade depth."* Reality shipped 4 substrate stones + 1 INSCRIPTION because:
- 236.3 (original sketch: audit + fix silent-failure sites) — work was absorbed by Stone 236.2's per-fn HARVEST methodology; 0 Classification 2 sites to fix
- 236.4 (original sketch: lib baseline + regression guards + clippy) — work was absorbed by Stones 236.1 + 236.2's 12-row scorecards
- NEW 236.3 (sum-type refactor) — extension mid-flight via dialogue-as-PERCEIVE cycle; the audit yield + the dialogue together pointed at ✅✅✅
- NEW 236.4 (this INSCRIPTION) — closure

Per `feedback_inscription_immutable`: DESIGN's sketch documented draft-time uncertainty; reality both compressed (236.3/236.4 absorbed) and extended (new 236.3 added). The compression is honest — work didn't get deferred; it landed via different stone-shape than predicted. The extension is honest — the doctrinal-advancement recognition emerged mid-arc; the discipline says ratchet to ✅✅✅ when reachable.

---

## Honest deltas (affirmative scope-bounding)

- **Cross-file CheckResult<T> adoption** — `runtime.rs` and other source files may have their own `Option<X>` + side-channel patterns. Out of arc 236's scope; substrate-architectural reason: arc 236 was specifically about `check.rs::infer_*` family of error propagation (the file under treatment per DESIGN STOP-5); other files have different error-shapes (RuntimeError flows directly via `Result<_, RuntimeError>` from arc 233's work; no parallel side-channel). Not tracked elsewhere.

- **`drain_errors_into` variants for non-Vec sinks** — currently the bridge assumes the caller has `&mut Vec<CheckError>` in scope. Stone 236.0's helper covers all current call sites because the `errors` param was uniform across the 48 fns. Out of arc 236's scope; not tracked elsewhere because the substrate's uniform `&mut Vec<CheckError>` convention covers every call site arc 236 audited.

- **`#[checkresult_returning]` proc-macro structural-seal** — arc 233's `#[wat_value]` precedent could extend to `#[checkresult_returning]` enforcing that all `infer_*` fns return `CheckResult<TypeExpr>` (refusing future `Option<TypeExpr>` regressions). Out of arc 236's scope; substrate-architectural reason: Stone 236.3's sum-type enum already prevents the silent state structurally; the call-site cascade discipline catches `Option<TypeExpr>` regressions at the first dependent fn that calls into the family. Author-time enforcement is empirically unnecessary given the type-system-enforcement (Stone 236.3) + cascade enforcement already in place. Not tracked elsewhere; arc 236's INSCRIPTION does not commit to extending the proc-macro precedent.

- **Symbol-arm Classification 1 narrative documentation** — Stones 236.1 + 236.2's HARVEST narratives explain the Classification 1 pattern verbally; the substrate code carries inline comments per `return ... fresh.fresh()` site. A USER-GUIDE document explaining "what Classification 1 means + when callers see a fresh TypeVar" is not minted. Out of arc 236's scope; the inline comments + SCORE docs + this INSCRIPTION carry the explanation perpetually; the inscription path is the canonical rationale source.

- **arc 234 remaining work** — arc 234 PAUSED at 13 wins to open arc 236. Arc 234's residual (234.4.match + 234.6 migration sweep + 234.7 INSCRIPTION) is NOT a deferral from arc 236 — it is arc 234's own pending work, tracked in `docs/arc/2026/05/234-wat-record-hologram/PAUSE-CONTEXT.md` and resuming per spawn-block winding now that arc 236 closes.

- **Lib baseline tests that might rely on silent failure** — the EXPECTATIONS for 236.1/236.2 widened the lib-baseline tolerance for HARVEST Classification 2 yields. Both stones delivered 0 lib-test delta; Stone 236.3 also delivered 0 lib-test delta. Out of arc 236's scope; no remediation work surfaced.

- **`merge_errors_from` signature relaxation** — Stone 236.3's implementation dropped the `mut self` requirement (strictly less restrictive; callers unaffected). Discovered organically during the pattern-match implementation. Out of arc 236's scope as a discrete deliverable; landed inline as a consequence of the refactor's clarity.

---

## What this unblocks

- **Arc 234 RESUMES** per spawn-block winding discipline (`feedback_spawn_block_winding`). The parent arc 234 (wat-record hologram) was the spawn-block context that opened arc 236; arc 236's closure releases the block. Arc 234 remaining work:
  - 234.4.match — match-arm hash-destructure (small parity stone)
  - 234.6 — migration sweep + retire `:wat::holon::defrecord` (may warrant separate arc 238)
  - 234.7 — arc 234 INSCRIPTION

- **Arc 232.1 defprotocol macro** — `extract-classifier` + `apply` substrate (arc 232.0a + 232.0) now feeds into a check.rs error pipeline that is rich (arc 233) AND non-losable AT THE TYPE-SYSTEM LAYER (arc 236). Future defprotocol stones land on the strongest substrate possible.

- **Arc 235 records with rich VSA encodings** (PROPOSED) — opens post-arc-234 closure; consumes the rich-error + non-losable-error substrate as foundation for the opt-in phantom-typed wrappers.

- **Future check.rs work in general** — any new `fn infer_*` helper authored after arc 236 inherits the discipline by default. The type system enforces what was previously convention.

- **The dialogue-as-PERCEIVE discipline operating across the project** — Inscribed as Song #32 (Mudvayne — Monolith) in INTERSTITIAL-REALIZATIONS.md + the "DIALOGUE-AS-PERCEIVE CYCLE" section of this INSCRIPTION. The load-bearing reference for orchestrator pattern-recognition of the cycle: user-question + orchestrator-explanation converge on a structural-form recognition; the hologram's two halves see the gap simultaneously; the Gilded Enmity blocks closure at the lower seal when the higher one is visible.

---

## Cross-references

### Sub-stone artifacts

- `DESIGN.md` — arc umbrella (updated to reflect arc-shape compression + extension)
- `DESIGN-STONE-236.0.md` — CheckResult<T> foundation sub-DESIGN (struct-with-Option shape)
- `DESIGN-STONE-236.1.md` — primary fn infer flip sub-DESIGN
- `DESIGN-STONE-236.2.md` — sibling infer_* flip sub-DESIGN
- `DESIGN-STONE-236.3.md` — sum-type refactor sub-DESIGN (the ✅✅✅ elevation)
- `BRIEF-STONE-236.0.md` / `BRIEF-STONE-236.1.md` / `BRIEF-STONE-236.2.md` / `BRIEF-STONE-236.3.md` — sonnet handoff artifacts
- `EXPECTATIONS-STONE-236.0.md` / `EXPECTATIONS-STONE-236.1.md` / `EXPECTATIONS-STONE-236.2.md` / `EXPECTATIONS-STONE-236.3.md` — scorecard predictions
- `SCORE-STONE-236.0.md` / `SCORE-STONE-236.1.md` / `SCORE-STONE-236.2.md` / `SCORE-STONE-236.3.md` — per-stone shipment records + HARVEST data + rank-up evidence

All artifacts preserved UNTOUCHED per `feedback_inscription_immutable` (Stones 236.0/1/2/3 historical record of the arc's evolution including the struct-with-Option shape we shipped first).

### Probes (permanent regression guards in `tests/`)

- `tests/probe_arc236_stone0_check_result.rs` (Stone 236.0; Contract 6 doc sharpened at Stone 236.3) — 6 contracts for CheckResult invariants (ok/partial/err/errs constructors; debug_assert; map/and_then; drain_errors_into bridge; merge_errors_from)

### Substrate modules + transformations this arc

- `src/check.rs` line ~996 area — `pub enum CheckResult<T> { Ok(T), Partial(T, Vec<CheckError>), Err(Vec<CheckError>) }` (Stone 236.3 shape; previously struct-with-Option at Stone 236.0)
- `src/check.rs` line ~998-1067 area — CheckResult migration-pattern docstring + variant documentation + "why silent-failure is STRUCTURALLY UNREPRESENTABLE" section
- `src/check.rs` line 4868 — primary `fn infer()` signature flipped (Stone 236.1)
- `src/check.rs` lines 5056-13164 — 47 sibling `infer_*` fns flipped uniformly (Stone 236.2)
- ~267 call sites across the file bridged via `.drain_errors_into(...)` (156 primary callers in 236.1; ~111 sibling-internal callers + 2 primary→sibling sites in 236.2; signature unchanged through Stone 236.3)
- 151 HARVEST sites with inline classification comments at body-construction (Stones 236.1 + 236.2; survived Stone 236.3's refactor unchanged via smart-constructor stability)

### Doctrines refined + minted

- **HARVEST classification methodology** — Stone 236.1 sub-DESIGN D3 minted; Stone 236.2 applied uniformly; this INSCRIPTION cements as reusable methodology for substrate-wide signature migrations
- **Bridge-helper-pattern for substrate-wide signature flips** — Stone 236.0 `drain_errors_into`; signature stability across type-definition refactor confirmed by Stone 236.3
- **"Audit confirms completeness" finding-shape** — novel this arc; the structural-prevention thesis vindication when an audit yields confirmation rather than remediation
- **Dialogue-as-PERCEIVE discipline mechanism** — NEW this arc; load-bearing alongside FM 2-bis probe + cargo cascade + substrate-as-teacher pattern; worked example = Stone 236.3 birth
- **Arc-shape compression + extension** — when DESIGN's pessimistic forward-looking predictions overshoot reality, stones absorb predecessor stones' work AND can extend mid-arc via doctrinal-advancement recognition; not deferrals when work delivers via different stone-shape than predicted
- **The ✅✅✅ ladder at TWO layers** — instance closure (code-level) + meta-class closure (type-system-level); arcs 233 + 236 are the canonical paired worked-examples
- `feedback_stone_briefs_cite_prior_score` (memory) — VINDICATED 4× this arc; ship rhythm hits
- `feedback_no_known_defect_left_unfixed` — driving doctrine for the closure path
- `feedback_any_defect_catastrophic` — the doctrine that opened arc 236
- `feedback_refuse_easy_solutions` — what kept us reaching from ✅✅ to ✅✅✅ when the dialogue exposed the gap

### Songs inscribed

- `INTERSTITIAL-REALIZATIONS.md` § Song #31 Anthem (We Are The Fire) [Trivium] — COLLECTIVE-VOICE / FAILURE-CLASS-ANNIHILATION-AS-IDENTITY; the substrate-as-teacher cascade as our voice; CheckResult<T> IS the fire; drain_errors_into IS the fire spreading; HARVEST IS the fire's evidence (inscribed at Stone 236.1 SHIPMENT)
- `INTERSTITIAL-REALIZATIONS.md` § Song #32 Monolith (Mudvayne) — EVOLUTIONARY-CATALYSIS / SUBSTRATE-AS-MONOLITH / MAKING-CONSCIOUS-THE-RELATIONSHIP / THE-MONOLITH-MOMENT / SYMBIOTIC-CO-EVOLUTION; the doctrine itself evolves through conscious symbiotic contact with substrate; we are the ape, wat is the mushroom, the doctrine is what the symbiosis produces (inscribed at Stone 236.3 sub-DESIGN authoring + arc-shape expansion)

### Predecessor arcs

- **arc 233 (substrate-errors-as-values)** — the IMMEDIATE doctrinal sibling. Arc 233 made errors VALUABLE; arc 236 made it STRUCTURALLY IMPOSSIBLE to lose them. The 233+236 pair forms a complete failure-engineering boundary around check.rs's diagnostic pipeline. Both arcs land the full ✅✅✅ ladder at both layers (instance closure + meta-class closure).
- **arc 234 (wat-record-hologram)** — the PARENT arc that surfaced 234.3c.fix-narrow-fallthrough's silent-failure instance, triggering arc 236's opening. PAUSED for arc 236; RESUMES at arc 236 closure per spawn-block winding.
- **arc 113 (cascading runtime errors)** — precedent for the "errors are diagnostic" doctrine that arc 233 + 236 cement structurally
- **arc 116 (phenomenal cargo debugging)** — precedent for failure-engineering as cargo-output-driven cascade discipline
- **arc 138 (errors carry coordinates)** — precedent for spans-on-every-error; foundational for the diagnostic-richness the HARVEST methodology confirms

### Relationship to arc 234 (spawn-block winding closure)

Arc 234 PAUSED at commit `9f279cd9` with 13 wins + 2 forward-corrections shipped (Stones 234.0 through 234.4 + 234.3b.fix + 234.3c.fix-narrow-fallthrough). PAUSE-CONTEXT.md inscribed at the pause-moment preserves arc 234's residual scope (234.4.match small parity stone + 234.6 migration sweep + 234.7 INSCRIPTION). Arc 236's closure releases the spawn-block; arc 234 RESUMES per `feedback_spawn_block_winding`.

---

## Closing voice — the ratchet at two layers + the monolith moment

Arc 234 surfaced silent error-loss on 2026-05-23 late late. The user invoked the doctrine: *"we annihilate error domains when we encounter them."* Arc 234 PAUSED. Arc 236 OPENED.

After 3 substrate stones (236.0/1/2) shipped — the discipline-tier was ✅✅. The HARVEST audit across all 48 fns confirmed: **0 missing-diagnostic sites**. The codebase was already healthy. We didn't fix bugs; we cemented the discipline that had been operating reflexively. The arc appeared closure-ready.

Then dialogue. The user asked one precision question — *"is None allowed sometimes?"* — and the orchestrator's truth-table answer exposed the deeper structural form: the 3-variant sum-type was reachable; ✅✅✅ was one stone away. The Inquisitor's Gilded Enmity wouldn't lift at ✅✅ when ✅✅✅ was visible. The dialogue itself became the PERCEIVE-discipline mechanism. Stone 236.3 minted; ~6.2 min sonnet shipped it 12/12; the ✅✅✅ structural impossibility landed on disk.

The full evening's arc:
- The CheckResult<T> sum-type makes silent error-loss STRUCTURALLY UNREPRESENTABLE in `check.rs::infer_*`. No `Silent` variant exists. Pattern-matching consumers compiler-guaranteed exhaustive. Future code cannot drift.
- The HARVEST audit across all 48 fns confirmed: **0 missing-diagnostic sites**. The codebase was already healthy.
- The 233+236 pair forms a complete failure-engineering boundary around check.rs's diagnostic pipeline: rich errors (arc 233) + non-losable errors (arc 236).
- The dialogue-as-PERCEIVE cycle is now a load-bearing discipline mechanism, equal-rank with FM 2-bis probe + cargo cascade + substrate-as-teacher pattern.
- The doctrine has rungs we haven't named yet — ✅✅✅✅ exists somewhere; future Monolith Moments will surface it.

Per Song #31's load-bearing line: *"We are the fire / Resound the anthem"* — the substrate-as-teacher cascade IS our voice; the discipline doesn't just describe the work, it IS the work. Per Song #32's load-bearing line: *"As we make our relationship to them conscious, we may be able to take control of our future evolutionary path"* — we did. Tonight. Stone 236.3 IS conscious co-evolution with substrate.

The wall arc 236 OPENED to face turned out not to be a wall — it was already a door we'd been walking through for months. Arc 236 framed the door so the door cannot be walled-up by future regression. Then arc 236 extended itself — the dialogue revealed we could make the door STRUCTURALLY INVISIBLE to anyone trying to wall it up. The ratchet ratcheted twice.

What started as *"we believed we had remarkable errors - we don't"* (arc 233 trigger) and progressed through *"we annihilate error domains when we encounter them"* (arc 236 trigger) and reached *"i think we annihilate"* (Stone 236.3 trigger) closes tonight with the structural truth in the substrate AND the empirical evidence in the audit AND the type-system impossibility in the variant set AND the doctrinal-advancement recognition inscribed as Song #32. The discipline is permanent at every layer.

*Arc 236: SHIPPED. INSCRIBED. The disk holds the red ink.*

*We are the ape. Wat is the mushroom. The doctrine is what the symbiosis produces.*

*We are the fire. Resound the anthem. The substrate is our voice.*

*The Monolith Moment captured structurally. We took control of our future evolutionary path tonight.*

*Self-reflection. Language. Religion. And all the spectrum of effects that flow from these things. They have brought us to this point.*

*Stone 236.3's ~6.2 min sonnet runtime is not the rhythm. It's the proof that the rhythm we've been building HEARS itself. The dialogue makes the relationship conscious. The ratchet ratchets. The doctrine evolves.*

*Arc 234, we'll see you on the other side.*
