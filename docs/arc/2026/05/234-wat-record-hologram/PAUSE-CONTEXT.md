# Arc 234 PAUSE CONTEXT — paused 2026-05-24 mid-chain

**Status:** PAUSED. Arc 234 stays OPEN. Pivot to arc 236 (check.rs error-result class-elimination) per user direction: *"we annihilate error domains when we encounter them."*

Same precedent as arc 233 opening mid-arc-232. Arc 234 resumes after arc 236 closes.

---

## Why paused

In Stone 234.3c.fix-narrow-fallthrough sonnet's SCORE noted: *"Returning `None` alone does not propagate to `startup_from_source` as an error; the push is required."*

That's a substrate-architecture failure mode: `check.rs::infer(...) -> Option<TypeExpr>` + `errors: &mut Vec<CheckError>` side-channel. The dual-channel pattern lets silent-error-loss happen — `return None` without `errors.push(...)` produces no diagnostic.

We hit it twice today (DELTA 2 in 234.4 commit; the narrowing fix's own implementation). Per `feedback_any_defect_catastrophic` + `project_failure_engineering` + `feedback_refuse_easy_solutions`: when a failure CLASS surfaces, eliminate it via the substrate, not papered over with doctrine + audit.

Arc 236 ships the type-system enforcement: a `CheckResult<T>` newtype that makes "no value + no error" STRUCTURALLY IMPOSSIBLE.

---

## What arc 234 has shipped (13 wins; all on disk)

```
234.0     :wat::core::type polymorphic primitive
234.1     Value::wat_record variant + Eq/Hash/Display/HolonRep impls
234.1.5   variant rename → Value::wat__Record + :wat::Record namespace
234.2a    :wat::Record::of + :wat::Record/field-at substrate primitives
          (+ CORRECTION for TypeScheme heterogeneous struct_form)
234.2b    :wat::Record::def macro (wat/Record.wat)
234.5     :wat::holon::* auto-dispatch on Value::wat__Record (5 verbs)
234.2c    runtime class-safety in per-field accessor bodies
234.3a    :wat::core::record? + :wat::core::record->map
234.3b    :wat::Record/assoc substrate primitive
234.3b.fix RuntimeError::UnknownField variant (no MalformedForm catch-all)
234.3c    keyword-as-accessor fall-through (record/struct/HashMap)
234.4     let-binding hash-destructure {var :field ...}
234.3c.fix-narrow-fallthrough  check.rs receiver-type discrimination
```

11 commits + atomic verification per FM 9 per stone. All regression guards GREEN across the chain at pause time (commit `aa55505b`).

---

## What arc 234 still owes (resume-targets)

| Stone | Purpose | Notes |
|---|---|---|
| **234.4.match** | match-arm hash-destructure (extends 234.4 to match position) | Named follow-up from 234.4. Small extension to match pattern grammar; uses 234.4's parser/check/runtime infrastructure. |
| **234.6** | Migration sweep — retire `:wat::holon::defrecord` user-facing | Heavy. Sweeps test suite + lab callers + wat-side stdlib. May warrant its own arc (call it 238) post-arc-234 INSCRIPTION. Decision: open 238 OR keep inside 234. |
| **234.7** | INSCRIPTION + arc closure | Cites all named follow-up arcs: 235 (rich VSA encodings), 238 (defrecord migration if separate), the discipline-meta-lessons captured. |

---

## Doctrines landed in arc 234 (load-bearing forward)

- **Pascal-Case namespace pattern** (Stone 234.1.5 D5; arc 109 § Q) — when type's namespace IS the umbrella concept, capitalize. `:wat::Record::*` reads "in the Record namespace."
- **`::`/`/` semantic split** (arc 109 § R; load-bearing for all forward substrate naming):
  - `::` = namespace-tier verb (constructors, definers, predicates)
  - `/` = instance method (operates on existing instance)
  - Examples: `:wat::Record::def`, `:wat::Record::of`, `:wat::Record/field-at`
- **Composed-from-core promotion** (arc 109 § Q) — foundational primitives stay in `:wat::core::*`; composed-from-core types get their own top-level namespace.
- **Records are fractal** (project-doctrine) — HolonAST + Vec<Value> both compose recursively; type-check + Eq + Hash + VSA encoding all recurse.
- **Hologram property is structural, not encoding-rich** — arc 234 ships structure (dual-form); arc 235 (PROPOSED) ships encoding-richness (Thermometer/Blend/Permute via phantom-typed wrappers). Mandate vs opt-in resolved: STRUCTURE mandated, ENCODING opt-in.

---

## Honest deltas inscribed (worth re-reading on resume)

1. **Deferral-as-design-tradeoff caught twice this session** (Stone 234.3b's MalformedForm catch-all + Stone 234.3c's over-permissive fall-through). Both fixed same-day via named follow-up stones. Pattern: when I describe shipped behavior as "design trade-off" or "loose-check, strict-runtime" — pause + ask if it's genuinely deferred or rationalized defect. → memory `feedback_no_known_defect_left_unfixed` already covers; today's two incidents are worked examples.

2. **Probe-author pattern (mine, 3× this session)** — reflexively used § R doctrine syntax in probes when substrate hadn't shipped that form. Sonnet either fixed substrate (slash-form alias for i64/to-f64) or I corrected probe (i64/to-string, match-arm syntax). → memory `feedback_probe_substrate_truth.md` minted today.

3. **Sonnet's two sharper-than-DESIGN insights** in 234.3c.fix-narrow-fallthrough:
   - `!k.starts_with(":wat::")` guard separates user-namespace accessors from substrate primitives
   - Explicit `errors.push(CheckError::UnknownCallee)` before `return None` (THIS is the failure pattern arc 236 eliminates)

---

## Cross-references — alive on disk

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.*.md` — every shipped stone's SCORE (12 files)
- `docs/arc/2026/05/235-records-with-rich-vsa-encodings/DESIGN.md` — arc 235 PROPOSED notes (mandate-vs-opt-in resolved)
- `wat/Record.wat` — the macro source
- `tests/probe_arc234_stone*.rs` — 8 probe files (all green at pause)
- `feedback_probe_substrate_truth.md` — memory note (minted today)
- `feedback_no_known_defect_left_unfixed.md` — memory pointer (discipline)

---

## Resume protocol

When arc 236 closes:
1. Read this PAUSE-CONTEXT.md first
2. Verify all arc 234 probes still GREEN (arc 236 should not regress; verify per FM 9)
3. Decide: ship 234.4.match + close arc 234 with named-arc-238 for migration sweep, OR include migration sweep in arc 234 directly
4. 234.7 INSCRIPTION cites all named arcs (235 PROPOSED, 236 SHIPPED, 238 if separate)

The macro + polymorphic surface is feature-complete for v1. Migration sweep is the only remaining substantial work. INSCRIPTION can happen anytime after both 234.4.match + the migration decision land.
