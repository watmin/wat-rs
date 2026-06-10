# SCORE — Stone 251.1 vigilatum ward (IN PROGRESS — stamp NOT yet earned)

**Status: 4 of the full guard cast; findings banked; the home is NOT stamped.**
The vigilatum is earned by combat against the WHOLE applicable book — this ward is
incomplete (a context-depth handoff, 2026-06-09). A rested guard finishes it.

`src/resolve/` is correct, green→green, clippy-clean, bug-fixed. Only the stamp waits.

## Casts run (4 of ~13 applicable)
- **intueri** ✓ — round-1; residual driven to **0 L1 / 0 L2** (commit `91c58342`).
- **purgare** ✓ — round-1; caught the **dead-fallback bug** (FIXED, commit `4b0dd67a`)
  — the `Type/member` fallback was structurally unreachable (reserved-prefix shortcut
  makes primary always-pass for `:wat::`). Latent type-member-symbol gap NAMED.
- **solvere** — VERDICT solid, **0 L1 / 1 L2**.
- **struere** — **2 L1 / 3 L2** reported (re-graded on weighing below).

## Findings + orchestrator weighing (the actionable set for the finish)

### KEYSTONE — the real one (solvere BRAID-1, struere F4 underlying)
**The quote-family boundary table is encoded TWICE** — `normalize.rs`
(`normalize_list`/`normalize_quasiquote_template`) and `walk.rs`/`quote.rs`
(`check_form`/`check_quasiquote_template`). The normalize.rs doc says "Mirrors
check_form's boundary logic exactly" — that names the braid. **They have already
drifted**: `normalize` lacks the `:wat::core::match`/`cond`/`matches?` pattern
boundaries that `walk` carries → a latent gap (namespaced symbols inside match-arm
PATTERNS are not boundary-protected by normalize). **FIX (decomplect):** extract one
`quote_boundary(head) -> Boundary` predicate (in `quote.rs` or a new `boundary.rs`)
that BOTH passes call; reconcile the match/cond asymmetry. This is the genuine L2 and
the highest-value finish item. struere's F4 (normalize "rewrites + validates" in one
signature) is the same coupling seen from the type side — the unification likely
dissolves both.

### Genuine mild L2 (fix in the finish)
- **struere F2** — `normalize_list` discards the span; the caller re-wraps
  (`normalize.rs:89`). Fold `normalize_list` into the `WatAST::List` arm of
  `normalize_form` (it's called once) so the boundary is structural.

### Re-graded to L3 / accept-with-reason (orchestrator weighing — grounded)
- **struere F4 (was L1)** — `normalize_symbol_refs` rewrites AND validates. This is the
  251.1b LOCATED-ERROR CONTRACT (the brief required "resolve to the entity it names;
  located error if not"), so validation is BY DESIGN; `ResolveError` is the honest error
  for an unresolvable ref. NOT a lie. (The deeper normalize/walk coupling is the KEYSTONE
  above.)
- **struere F6 (was L1)** — `resolve_references` two-pass top-level `use!`-scan scope
  assumption invisible in `&[WatAST]`. **PRE-EXISTING** (verbatim-lifted from resolve.rs,
  not introduced by 251.1); single caller (freeze.rs) always passes top-level residue.
  Cheap honesty fix: a doc line stating "top-level forms only," or a `TopLevelForms`
  newtype — future nicety, not a 251.1 defect.
- **struere F1 (L2)** — `normalize_form` threads `&mut Vec<errors>` rather than returning
  `Result`. DELIBERATE: multi-error accumulation (report ALL unresolved refs together),
  mirroring `resolve_references`' own accumulator. `Result` would lose that. Accept; or
  add a `rune:struere`.
- **struere F5 (L2)** — `UnresolvedReference.context: &'static str` constrains to literals.
  PRE-EXISTING; fine for the current fixed contexts; widen to `Cow<'static,str>` only if
  dynamic context is ever needed. Doc the constraint or accept.
- **struere F3** — `format!("{}::", decl)` per `:rust::*` head: a temperare concern (perf),
  not struere; pre-existing; tiny.

## The finish (rested guard)
1. recolligere → read this SCORE + the round-1 close commits (`4b0dd67a`, `91c58342`).
2. **Fix the KEYSTONE** (extract `quote_boundary`, reconcile match/cond asymmetry) +
   struere F2 (span); doc-fix F6; rune/accept F1/F5.
3. **Cast the rest of the book** (within reason): sequi, conformare, exigere, perspicere,
   cernere, temperare, the test-wards (vocare/complectens/probare), circumspicere LAST.
   Skip the inapplicable (secare — no parallelism; mora — no waits).
4. Weigh each against the disk → drive to 0 L1 / 0 L2 → L3 accepted-with-reason → earn the
   vigilatum stamp in `mod.rs` (replace the placeholder).
