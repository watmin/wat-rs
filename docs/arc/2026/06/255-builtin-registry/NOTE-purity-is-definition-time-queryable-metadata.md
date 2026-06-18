# NOTE — purity is a DEFINITION-TIME property; it must be queryable metadata (the arc-283 sweep demanded it)

> Surfaced 2026-06-17 by the arc-277 sweep (the self-hosted linter rewriting its own corpus). Builder:
> *"we need to make a note that we declare a function pure or not at definition time — there's an arc to
> make all intrinsics have queryable metadata — this just demanded that."* That arc is THIS one
> (255 — builtin-registry / queryable intrinsic metadata).

## What demanded it

The sweep's concat→format auto-fix rewrote `(string::concat …)` calls into `(:wat::core::format …)`.
`format` is a **macro** (`wat/core.wat:545`). Many of the rewritten concats live inside **defmacro
bodies** (building keyword names at EXPAND time — `(concat fqdn-str "::Op")` etc.). The macro-eval
purity gate (arc 249 stone 249.2b-i, default-deny F5) refused them at load:

> `keyword head :wat::core::format refused at macro expand time — not on the pure-combinator allow-list`

The whole stdlib failed to load (deftest 0/263). Reverted; nothing shipped.

## The principle the failure names

**Whether a callable may appear in an expand-time (macro-eval) position is a property declared at its
DEFINITION** — `format` is a macro (never expand-time-legal as a value); `string::concat` is pure-total
(expand-time-legal); a runtime `defn` is neither. Today that property is scattered and implicit: a
hand-maintained Rust `is_pure_total` allow-list (`src/macros/eval.rs`) for intrinsics, plus the
macro/defn/defclause distinction for user forms. **There is no single queryable fact a TOOL can ask:
"is `:wat::core::X` usable at expand time? at runtime? is it a macro?"**

A codemod that introduces a call (the concat→format fix; any future fix) MUST be able to query the
target POSITION's purity context AND the introduced callable's purity class, to know the rewrite is
legal. The lint/RETE engine (arc 277/278) is the first hard consumer; it cannot stay correct on a
guessed/implicit purity model.

## What this arc should carry (the ask)

- **Definition-time purity/expand-time-legality as a first-class, queryable metadatum** on every
  intrinsic + user callable in the builtin registry — `(metadata-of :wat::core::X)` answers
  `{:kind macro|fn|intrinsic, :pure-total bool, :expand-time-legal bool, …}`.
- The Rust `is_pure_total` allow-list becomes a *projection* of this registry, not a parallel
  hand-list (extirpare: one source of truth; the drift between the allow-list and reality is the
  failure class).
- Consumers (the RETE engine, codemod fixes) QUERY it rather than re-deriving — see arc 278's
  output-contract + the concat→format fix's macro-context gate (arc 277), both of which need this.

(Companion requirement, captured in arc 277 + 278: the detection rule also needs to know whether the
form under inspection is ITSELF in an expand-time position — "am I inside a defmacro body?" — which is
the context half; this note is the callable-class half. Both are needed to gate a macro-introducing fix.)
