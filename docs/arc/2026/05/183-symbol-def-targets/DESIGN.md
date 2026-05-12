# Arc 183 — Symbols as `def` targets (substrate enabler for opt-in short-name aliasing)

**Status:** stub opened 2026-05-12 per user direction.

## Motivation

> *"I want to be able to vend an optional crate — 'wat-short-names'
> or something — being able to alias :wat::core::+ => + ... users
> can opt into convenience forms but they are always backed by
> fqdn keywords"*
>
> *"right now we only allow them in let but we've only allowed
> keywords (so far) because wat's core tooling is fqdn keywords"*

Currently `def` / `defn` / `define` accept ONLY keyword targets
(`:wat::core::+`, `:my::app::compute`, etc.). The FQDN-keyword
constraint is intentional — the LLM-first design (memory:
`project_wat_llm_first_design.md`) deliberately rejects synonyms
in the core substrate to keep the path-of-least-resistance the
LLM-friendly path.

This arc opens a SUBSTRATE ENABLER for a separate optional layer:
allow symbols (bare names like `+`, `map`, `inc`) as def targets
when the def's RHS is itself a FQDN keyword (i.e., an alias —
not a fresh definition).

The orthogonal split:
- **Core substrate (always FQDN keyword)**: `(:wat::core::def :my::app::compute (:wat::core::fn ...))`
- **Opt-in short-name aliasing (this arc enables)**: `(:wat::core::def + :wat::core::+)` — symbol `+` aliases the FQDN keyword
- **Opt-in crate (`wat-short-names` or similar)**: registers a curated set of these aliases at load time; users `(:wat::load-crate! :wat::short-names)` to pull them in

## The doctrine tension (worth surfacing)

Memory entry `project_wat_llm_first_design.md`:
> *"brutal honesty + minimal forms + one-canonical-path-per-task
> is engineered pedagogy for AI co-authors. Reject synonym
> features. Force naming."*

This arc adds an OPT-IN synonym layer. The defense:
- Default substrate stays FQDN-only (no opt-in = no synonyms)
- The opt-in crate is a CONSUMER preference, not a core feature
- LLM-authored code naturally reaches for FQDN; opt-in crate is for
  HUMAN-authored code where brevity matters
- The canonical form remains the FQDN-keyword; aliases are pointers,
  not independent definitions

If the discipline holds in practice (LLM code stays FQDN even with
the crate available), the split is honest. If LLM code starts
mixing aliases, the LLM-first principle is broken; revisit.

## Sketch (placeholder; user fills the design)

TBD. Likely shape:

1. **Parser**: symbols already lex as `WatAST::Symbol`; the
   substrate constraint is at the def's NAME-position type check
2. **Substrate enabler**: `parse_define_form` / `try_parse_fn_shape_def`
   accept `WatAST::Symbol` in name position; resolve-pass treats
   the symbol as a top-level binding pointing at the RHS keyword
3. **Resolve semantics**: when a call head is a symbol (e.g., `(+ 1 2)`),
   resolve looks up the symbol's bound FQDN-keyword and dispatches
4. **Constraint**: the RHS of a symbol-targeted def MUST be a
   FQDN-keyword (alias-only); not a fresh fn definition. Enforced
   at substrate level; gives the LLM-first guarantee teeth.

Open questions for the design:
- Namespace collision: what if two short-name crates alias `+` to
  different FQDN keywords?
- Shadow vs override: precedence rule
- Help-reflection: does `(help +)` show the FQDN it aliases?
- Print-back: does the substrate render values with FQDN or alias?
  (Probably FQDN — keep the substrate's output canonical)

## Cross-references

- arc 157 (def form) — current keyword-only target
- arc 166 (defn form) — same constraint
- arc 178 (primitive Type/fn shape) — defines what gets aliased
- arc 109 § kill-std + FQDN doctrine — the constraint this orthogonally extends
- arc 168 (let flat shape) + arc 159 (untyped let bindings) —
  current symbol-in-let precedent (where symbols are already allowed
  as binding names)
- memory: `project_wat_llm_first_design.md` — the LLM-first tension
- memory: `feedback_fqdn_is_the_namespace.md` — FQDN doctrine
