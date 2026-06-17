# DESIGN-STONE 260.1 — user-fn keyword arguments (the reorder mechanism)

> Opened 2026-06-17. First stone of arc 260. Grounded against HEAD `b644c5c6`. Builds on the arc's
> GROUNDING + PROBE FINDINGS (DESIGN.md): kwargs are a real new feature; user fns retain param NAMES
> (`func.params` at eval, `sym.functions[path].params` at check), so the reorder is feasible on existing
> data. Intrinsics (no names) are 260.2 — NOT this stone. Gate probe committed: `tests/probe_arc260_keyword_args.rs`.

## What this stone delivers

A call to a **user `defn`** may pass its arguments as `:param-name value` pairs in ANY order; the call
reorders to positional by the callee's declared param names, then checks + binds exactly as today.
`(:user::sub :b 3 :a 10)` → reorder → `(sub 10 3)` → 7. Legibility at the call site (the arc's thesis),
on the data that already exists.

## THE contract decision — disambiguation (keywords are first-class VALUES in wat)

`(f :fast)` is ambiguous: `:fast` could be a keyword *value* passed positionally, or a *label*. wat has
keywords as values, so this must be resolved, not hand-waved. **Pinned rule (all-or-nothing keyword mode):**

> A call is in **keyword mode iff its argument list is entirely `:name value` pairs whose `:name`s are
> exactly the callee's fixed param names — every param named once, no unknowns, no duplicates.** Otherwise
> the call is **positional** (unchanged). No mixed positional+keyword in this stone.

- **Unambiguous by construction.** A positional call that passes a keyword *value* (e.g. the lone
  `serve <- :keyword` param, `spawn.wat:195`) is NOT all-`:name value`-pairs-covering-all-params, so it
  stays positional. Keyword-valued args are nearly nonexistent (1 in the tree); the rule leaves them
  untouched.
- **Validation is the kwargs-map discipline we already shipped** (defservice opts): unknown name → named
  error ("`:f` has no parameter `:x`"); missing a param / duplicate name → named error. Raise the bar:
  the user's mistake is reported directly, never silently mis-bound.
- **Clojure-faithful + honest:** the label IS the param name, matched structurally; reorder happens
  BEFORE type unification (check) and BEFORE binding (eval), so the type story and arity are unchanged —
  kwargs are pure surface sugar over the positional core, no new runtime/type semantics.
- **Out of scope (affirmative cut):** mixed `(f 1 :b 2)` (positional + trailing kwargs, Python-style) —
  a later stone if wanted; all-or-nothing is the honest floor. Intrinsics — 260.2 (they have no names).
  Rest/variadic params interacting with kwargs — 260.x if a caller needs it.

### Four-questions on the disambiguation rule
- **Obvious? YES** — `(f :a 1 :b 2)` reads as exactly what it is; the all-or-nothing rule is one sentence.
- **Simple? YES** — one detector ("all args are `:param-name value` pairs covering the params") + one
  reorder, applied at the two resolved-call sites. Pure surface sugar; no new type/runtime concept.
- **Honest? YES** — no ambiguity (keyword-valued positional stays positional); every malformed kwarg call
  is a named error; reorder-before-unification keeps types/arity truthful.
- **Good UX? YES** — the call site labels itself; mistakes are named. (Mixed-mode is a future nicety, not
  a correctness gap.)

## The mechanism (where it lands)

Parse stays positional (`parse_list_body` → `Vec<WatAST>`; the parser can't know the callee's signature).
The reorder is a small pass at each **resolved** call site:

1. **Check** (`src/check.rs` call-inference): when inferring a call `(f a…)` to a user fn, look up
   `sym.functions[path].params` (the names — present, NOT in the scheme `TypeScheme.params` which is
   types-only). If the args are keyword-mode, reorder to positional by those names (validate; named error
   on mismatch), THEN run the existing positional arity+unification against the scheme.
2. **Eval** (`src/runtime.rs` `apply_function` ~17990, and the user-fn call path): same detect + reorder by
   `func.params` BEFORE `func.params.iter().zip(args)` binding.

The detector + reorder is ONE helper used by both sites (a pure `(params: &[String], args) -> Result<Vec<positional>, KwargError>`), so check and eval agree by construction.

## STOP triggers (for the brief)
1. STOP if the check-side call-inference can't reach `sym.functions[path].params` at the call site (the
   grounding says it can — `sym.functions` is iterated in check; confirm the call-inference has `sym`).
2. STOP if reorder-before-unification breaks generic/`∀T` inference (it shouldn't — reorder is purely
   positional rearrangement before the existing unifier runs; confirm with a generic-fn kwarg probe).
3. STOP if the all-or-nothing detector misfires on an existing positional call (it must not change ANY
   current call's meaning — the full suite is the guard).

## Gate
- `tests/probe_arc260_keyword_args.rs` (the committed RED probe) GREEN, `#[ignore]` removed.
- Add: a kwarg call with an UNKNOWN name → named error; a kwarg call MISSING a param → named error; a
  positional call with a keyword VALUE arg still works (no misfire); a generic fn called with kwargs works.
- lib 929/36, nursery 893/4 (zero new — no existing positional call changes meaning).

## Open for builder ruling (syntax is yours)
The disambiguation rule above (all-or-nothing keyword mode, opt-in by *usage* not by signature
declaration) is the pinned contract. The alternative is **signature-declared opt-in** (Clojure
`& {:keys [...]}` — a fn must mark itself kwargs-callable). I recommend usage-detected all-or-nothing: it
needs no new signature syntax, the ambiguity is nil in practice (1 keyword-valued param), and it labels
EVERY user call site (the thesis) rather than only opted-in fns. Confirm or override before the brief.
