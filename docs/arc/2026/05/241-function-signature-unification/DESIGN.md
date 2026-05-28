# Arc 241 — function-signature parser unification (one canonical argspec parser)

**Status:** OPEN 2026-05-27 night. Arc 237 PAUSED at 237.8b (DESIGN + probe committed `49e2e13b`); resumes when 241 closes.

**Origin:** Stone 237.8b's FM-2-bis probe (`tests/probe_arc237_8b_defclause_arithmetic.rs` Gate 1) surfaced that `defclause`'s argspec parser doesn't support `&` rest-binders. The user pushed: *"why isn't this using the tooling that fn and defn use?"* — the right question.

The dig confirmed: **the substrate has FOUR copies of the same argspec-parser logic** — each independent code path; subtly different in error-message wording and per-site invariants; none currently support `&`. The code itself names the duplication. Per the wat philosophy (one canonical path; remove options), this is a failure class to eliminate at the structural level — not "add `&` to defclause's parser" but "consolidate to ONE parser; extend it ONCE; all consumers inherit."

This is exactly the 109 endeavor's pattern — substrate consolidation to a single canonical surface.

---

## The failure class

```
Today, the substrate has FOUR parsers for [name <- :Type ...] arg-vector triples:

  src/runtime.rs:6750  parse_fn_signature                  (fn — runtime)
  src/check.rs:15205   parse_fn_signature_for_check        (fn — check)
  src/check.rs:15258   parse_fn_signature_for_check_diag   (fn — check, diagnostic variant)
  src/runtime.rs:6880  parse_defclause_args                (defclause — runtime)
```

The code itself names the duplication — `parse_defclause_args` (runtime.rs:6877) docstring:

> *"Reuses the same `name <- :T` triple shape as `parse_fn_signature` but enforces the defclause binding contract (arc 159/169/234): the name slot MUST be a Symbol (not a literal integer, keyword, etc.)."*

DESCRIPTION-level reuse + CODE-level COPY. The author knew. It happened anyway.

### What the failure class allows

Four parsers means four places where:
- The form's accepted shape can diverge silently (e.g., one supports `&`, another doesn't — exactly today's state)
- Error messages drift (`"fn arg-vector triple at position N"` vs `"defclause arg-vector triple at position N"` — same problem, different wording per parser)
- Future extensions land asymmetrically (a destructuring pattern added to fn's parser; defclause's parser doesn't know about it; consumer behavior bifurcates)
- Per-binding-site invariants are buried in scattered places (defclause's "name must be Symbol" check vs fn's silent-acceptance of non-Symbols)

The CLASS of failure: **parser divergence across binding sites**. Consumers cannot predict which arg-vector forms work where. The substrate accepts a form that the next binding site silently rejects. LLM co-authors generate code that works in one site and breaks in another. The institutional pattern of "we'll keep them in sync by convention" fails the moment the second feature lands asymmetrically.

State to make unrepresentable: **two binding sites accepting different arg-vector forms**.

### What wat's philosophy demands

> *"a key philosophy in wat is removal of options — there's only one way to do things"*

Today there are FOUR ways the substrate parses arg-vector triples. The philosophy says ONE. The current state IS a violation of the philosophy hidden behind code-comment "reuses" that aren't.

---

## The class-elimination strategy

Mint ONE canonical parser. Every binding site routes through it. Extensions land ONCE.

```rust
// src/argspec.rs (new module)

/// The flat arg-vector triple form used at every binding site:
///   [name <- :Type  name <- :Type ...]
///   [name <- :Type  ...  & rest <- :Vector<T>]
///
/// One canonical parser. Every consumer (`fn`, `defn`, `defclause`,
/// `defrecord`, `let`, future binding sites) routes through here.
/// Extensions to the form land ONCE.
pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    pub rest_param: Option<(String, TypeExpr)>,  // None = fixed-arity; Some = & rest
}

pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,  // per-site invariants (e.g., "name must be Symbol" for defclause)
) -> Result<ArgSpec, RuntimeError> { ... }

pub struct ParseOptions {
    /// If true, name slot MUST be a Symbol (defclause + defrecord enforce).
    /// If false, name slot can be any binding-pattern (fn allows future destructuring).
    pub name_symbol_only: bool,
    /// If true, `& rest <- :Vector<T>` is permitted as the final triple.
    pub allow_rest_binder: bool,
}
```

The constructors / options enforce the invariant: **no binding site CAN have a divergent parser** because there's only ONE; per-site differences live in `ParseOptions` (a small enum of choices that's explicit, not implicit).

Authors of new binding sites cannot accidentally duplicate the parser because the only way to parse arg-vector triples is to call `parse_argspec_triples`. The duplication failure class is structurally unrepresentable.

This is the failure-engineering pattern at the parser layer: don't avoid divergence, never construct the situation that allows it.

---

## Migration scope (audit-first; stones investigate per-site)

Probable touch sites:
- `src/runtime.rs:6750` `parse_fn_signature` — migrate to call `parse_argspec_triples`
- `src/check.rs:15205` `parse_fn_signature_for_check` — migrate
- `src/check.rs:15258` `parse_fn_signature_for_check_diag` — migrate (diagnostic variant; preserves richer error reporting via a HOF if needed)
- `src/runtime.rs:6880` `parse_defclause_args` — migrate
- **Other binding sites** — `defrecord` (arc 227.2 v3), `let` (arc 159), `defservice`, future ones — audit; likely each has its own copy or close variant. Stone 241.0 (AUDIT) finds them all.

Each site keeps its per-site invariant via `ParseOptions`. Behavior pre-migration is preserved at each site; the substrate-as-teacher cascade surfaces any drifts during the sweep.

---

## Stone sketch

- **241.0 — AUDIT.** Enumerate every argspec-parser copy in the substrate. Read each one; note: per-site invariants (name-must-be-symbol? type-required? arity-min?); per-site error-message wording; per-site special-cases. Produce `AUDIT.md` listing all copies + their differences + the consolidation plan. No substrate change.
- **241.1 — Mint canonical `parse_argspec_triples` + ArgSpec/ParseOptions types** in new module `src/argspec.rs`. Includes unit tests covering the basic shapes + per-site options. NO migration yet; the new parser stands alongside the old ones. Verify lib stays 834/0.
- **241.2 — Migrate fn parsers** (runtime + check + check-diag). All three `parse_fn_signature*` route to canonical. Behavior preserved exactly (probe baseline matches HEAD); error-message wording may align to the canonical voice.
- **241.3 — Migrate defclause parser** (`parse_defclause_args`). Route to canonical with `name_symbol_only: true`. Existing 237.x probes pass identically.
- **241.4 — Migrate other binding sites** discovered in 241.0 (defrecord, let, defservice, etc.). One-by-one; cascade-aware.
- **241.5 — Extend canonical with `&` rest-binder support** (the new capability). All binding sites that opt in via `ParseOptions::allow_rest_binder = true` get `& rest <- :Vector<T>` for free. The probe `probe_arc237_8b_defclause_arithmetic.rs` Gate 1 flips green automatically. **This unblocks 237.8b.**
- **241.6 — INSCRIPTION + arc closure** + memory mint (`feedback_argspec_parser_canonical` — the doctrine; "one canonical parser; per-site invariants via ParseOptions; substrate has ZERO duplicate parsers post-241").

Substrate work: 241.0–241.5 (6 stones). Closure: 241.6. Total: 7 stones.

**Could compress** (per the 236 pattern where 236.3 + 236.4 absorbed into 236.2's HARVEST methodology):
- 241.2 + 241.3 + 241.4 might bundle if the migration is identical per-site mechanical (call-site rewrite + delete old fn). Sonnet's flight will tell us.

---

## What this protects forward

After 241 closes:
- Authors cannot duplicate argspec parsers — type system + module structure enforces (only one entry point exists)
- Per-binding-site invariants live in one place (`ParseOptions`), explicit + auditable
- Future argspec-form extensions (destructuring? default values? optional type annotations?) land ONCE and apply to all binding sites uniformly
- `feedback_argspec_parser_canonical` becomes the doctrine; LLM co-authors mirror the canonical shape; substrate stays ONE parser permanently
- Arc 237.8b unblocks (and via 237.8b, the recipe-lock for arithmetic + ordering grids; via 237.8c, the equality grid; via 237.9, the recipe doctrine inscribed)

This is the foundation that makes the recipe-lock (per-Type primitives + wat-defclause polymorphic surface) shippable cleanly. The recipe ASSUMED defclause supports `&`; the substrate didn't; the assumption forced us to surface the duplication; the duplication-fix becomes the foundation.

Per the trap-door-build-the-dependency discipline: the dig surfaced the constraint; the next move is BUILD the missing piece, not work around it.

---

## Sequencing — arc 237 PAUSES until 241 closes

Per spawn-block winding (`feedback_spawn_block_winding`): arc 237 was active; arc 237.8b spawned the dependency (defclause `&` support); the dependency surfaced a deeper unification need; arc 241 spawns to handle it; arc 237 PAUSES; arc 241 closes; arc 237 RESUMES at 237.8b (Gate 1 flips green automatically via 241.5's extension).

**Arc 237 PAUSE context** (`docs/arc/2026/05/237-polymorphism-consolidation/PAUSE-CONTEXT.md` mints with this arc's open):
- 237.7 (collection-ops intrinsic phase) COMPLETE
- 237.8a (arithmetic + comparison HARD CUT under THE DECISION) SHIPPED `154ca713`
- 237.8b sub-DESIGN + FM-2-bis probe COMMITTED `49e2e13b` (probe surfaces the blocker)
- Awaiting 241 closure → resume at 237.8b BRIEF
- Eventual chain: 237.8b (recipe-lock) → 237.8c (equality grid) → 237.8d (DispatchRegistry HARD CUT) → 237.9 (INSCRIPTION)

---

## STOP triggers (arc-level)

- **holon-rs touched** — frozen substrate; out of scope
- **arc 237 probes regress** during migration (especially the 237.7c assoc probe + 237.8a no-implicit-coercion probe) — the parser unification must NOT change behavior at any existing binding site
- **lib baseline < 834** — strict-no-regression
- **scope creep beyond argspec parsing** — e.g., changing fn's syntactic shape, adding destructuring patterns, etc. — those are FUTURE work atop the canonical parser; 241 ONLY unifies what exists today + adds `&` rest-binder (the one capability 237.8b needs)
- **duplicate parser created during migration** — explicit anti-pattern; reject and re-brief
- **silent error-message drift** — every site's error-message wording is preserved or aligned to a CANONICAL voice; no per-site noise

---

## Calibration (arc-level)

**Target:** 5–7 stones; one substantial multi-stone block. Each stone ~Mode-A (20-60 min). Total session: probably one full work-block.

**Confidence:** medium-high. The mint stone (241.1) has clear structural shape; the migration stones (241.2-4) are mechanical per-site call-site rewrites; the extension (241.5) is small surface (parser branch on `&` + binding-to-Vector<T> at eval); the audit (241.0) bounds the unknown.

Single-stone calibrations TBD per stone.

---

## Doctrinal frame — failure-engineering at the parser layer

Per `/home/watmin/work/holon/scratch/FAILURE-ENGINEERING.md`:

> *"Failure engineering insists on the second [level of fix]. The failure isn't 'this specific case panicked.' The failure is 'a class of inputs / states / interactions can produce this kind of panic.' The fix isn't 'make this case stop panicking'; the fix is 'make this CLASS of panic structurally impossible.'"*

The failure class here: **parser divergence across binding sites**. The structural-impossibility fix: **one canonical parser; module privacy prevents alternative entry points; per-site invariants explicit via ParseOptions**. Post-241 the failure class cannot manifest because the situation that produces it cannot be constructed.

Per the user's framing:
> *"a key philosophy in wat is removal of options — there's only one way to do things"*

Post-241: one way to parse arg-vector triples. The philosophy is materialized in the substrate, not in convention.

---

## Cross-references

- `/home/watmin/work/holon/scratch/FAILURE-ENGINEERING.md` — the discipline that drives this arc
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — the recent canonical class-elimination arc; this arc's shape mirrors 236's
- `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.8b.md` — the stone that surfaced the blocker
- `tests/probe_arc237_8b_defclause_arithmetic.rs` Gate 1 — the empirical evidence (RED at HEAD; flips green post-241.5)
- `feedback_wat_llm_first_design` — one-canonical-path principle; this arc materializes it for argspec parsing
- `feedback_refuse_easy_solutions` — discipline driving "consolidate first, extend second" over "just add `&` to defclause"
- `feedback_trap_door_build_the_dependency` — the dig surfaced the duplication; the next move is BUILD, not work around
- `feedback_spawn_block_winding` — arc 237 PAUSES until 241 closes
- `feedback_sonnet_writes_substrate` — orchestrator briefs; sonnet executes per stone
- arc 109 (kill-std) — the LARGER substrate-consolidation work this arc is a focused piece of
