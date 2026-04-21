# Arc 009 — Names Are Values — Backlog

**Opened:** 2026-04-21. Small substrate closure.
**Motivation:** arc 006's `with-state` slice surfaced a real language
gap — a registered define's keyword-path in value position produced
a `Value::wat__core__keyword` literal, not a callable value. Users
had to wrap every named define in a pass-through lambda to satisfy
`:fn(...)`-typed parameters. The lambda carried zero information the
name didn't — pure ceremony.

`:wat::kernel::spawn` already accepted function-by-name; stdlib and
user combinators (map, filter, with-state, ...) did not. The
asymmetry was historical, not principled.

---

## Tracking

| Item | Status | Commit |
|---|---|---|
| `eval` on `WatAST::Keyword` — lift registered function paths to `Value::wat__core__lambda` | **done** | this slice |
| Type-checker `infer` on keyword in expression position — lift registered schemes to `:fn(...)->Ret` | **done** | this slice |
| Integration tests — named define as value, polymorphic instantiation, higher-order use | **done** | this slice (`tests/wat_names_are_values.rs`, 5 tests) |
| Update `wat/std/stream.wat` chunks rewrite to pass `chunks-flush` by name | **done** | this slice |
| Update `wat-tests/std/stream.wat` with-state tests | **done** | this slice (6 deftests, passing) |

---

## Decision log

- **2026-04-21** — Scope. Close only the user/stdlib-define lift path
  in runtime. Primitives (kernel/algebra/config/io) that have schemes
  but no `sym.functions` entry can still pass the type check — no
  restriction there — but at runtime they remain call-only. If a
  caller surfaces that wants to pass a primitive as a value, we
  synthesize a Function wrapper then. Stdlib-as-blueprint: ship the
  caller-demanded path; defer speculation.
- **2026-04-21** — Symmetry with kernel. `:wat::kernel::spawn` has
  always accepted function-by-name at its first arg via its own
  `infer_spawn` path (check.rs). This arc generalizes that pattern
  to every `:fn(...)` parameter position, so the asymmetry dissolves.
- **2026-04-21** — No design doc. The change is small substrate with
  one honest motivation; BACKLOG + INSCRIPTION is enough scaffolding.
  Same shape arc 005 used.

---

## Why this matters beyond with-state

- `(:wat::std::stream::map source :my::transform)` — no lambda
- `(:wat::std::stream::filter source :my::pred)` — no lambda
- `(:wat::std::stream::with-state stream init :my::step :my::flush)` — no lambdas
- Any future higher-order combinator — same

A named define IS a function value, the way every serious language
treats it (Rust `fn foo` passed to `.map(foo)`, Clojure `(map my-fn
coll)`, Haskell `map myFn xs`, Scheme `(map my-proc lst)`). This arc
brings wat to the same line.
