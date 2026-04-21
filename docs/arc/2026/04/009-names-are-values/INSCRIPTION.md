# Arc 009 — Names Are Values — INSCRIPTION

**Status:** shipped 2026-04-21.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the gap and its resolution.
**This file:** completion marker.

---

## What shipped

Two sites changed; everything downstream composes.

### `src/runtime.rs` — `eval` on `WatAST::Keyword` (value position)

Before: a keyword always evaluated to `Value::wat__core__keyword`
(except `:None`, which was already special-cased as the `:Option<T>`
nullary constructor).

After: if the keyword is a registered path in `sym.functions`, it
lifts to `Value::wat__core__lambda(func.clone())`. If not, it stays
a keyword literal.

```rust
if let Some(func) = sym.get(k) {
    return Ok(Value::wat__core__lambda(func.clone()));
}
Ok(Value::wat__core__keyword(Arc::new(k.clone())))
```

### `src/check.rs` — `infer` on `WatAST::Keyword` (expression position)

Before: every keyword inferred to `TypeExpr::Path(":wat::core::keyword")`.

After: if the keyword resolves to a `TypeScheme` in the check
environment, the scheme instantiates — fresh unification variables
for each type parameter — and the result is a `:fn(params)->ret`
TypeExpr. The keyword becomes callable at both ends of the pipeline.

```rust
WatAST::Keyword(k) if env.get(k).is_some() => {
    let scheme = env.get(k).expect("guard").clone();
    let (params, ret) = instantiate(&scheme, fresh);
    Some(TypeExpr::Fn { args: params, ret: Box::new(ret) })
}
```

The existing `infer_spawn` path (check.rs line 1317) that did this
for `:wat::kernel::spawn`'s first argument stays unchanged — spawn's
handling was ad-hoc, specific to one primitive. This arc generalizes
the same pattern to every expression position, and the spawn path
now reads as a specialization of the general rule.

### Integration tests — `tests/wat_names_are_values.rs`

Five tests, all pass first try:

- `named_define_is_a_function_value` — `:my::double` bound to a
  `:fn(i64)->i64` let* slot, called via the binding, produces the
  expected arithmetic result.
- `named_define_passes_to_higher_order_fn` — user-defined
  `:my::apply-twice (f x)` accepts `:my::inc` by bare name; no
  wrapper.
- `polymorphic_named_define_instantiates_at_use_site` —
  `:my::identity<T>` passed to `:fn(i64)->i64` slot; T instantiates
  to i64 at the call site.
- `unregistered_keyword_still_a_literal` — a keyword that is not a
  registered define stays `:wat::core::keyword`. The lift is
  conditional on registration.
- `named_define_as_stream_map_fn` — canonical use case:
  `(:wat::std::stream::map source :my::double)` — no lambda wrapper.
  Stream produces doubled values end-to-end.

### Downstream simplifications

- `wat/std/stream.wat` — the `chunks` rewrite on `with-state`
  (arc 006) now passes `chunks-flush` by bare name. The `chunks-step`
  case still uses a lambda (because it closes over `size` — genuine
  information, not ceremony). One lambda retired from the chunks
  surface.
- `wat-tests/std/stream.wat` — the with-state dedupe and
  buffer-all tests pass their step/flush lambdas via let\*-bound
  names; the `:None` initial state binds through a typed let\*
  binding rather than the attempted call-form that sparked the
  original runtime error.

---

## What this fixes, named

The asymmetry between `:wat::kernel::spawn` (accepts function-by-name
at its first arg) and every other `:fn(...)`-typed parameter
(required lambda wrappers) was historical, not principled. Both
positions now accept a bare keyword-path reference.

Every stdlib combinator with a function parameter is cleaner:

```
(:wat::std::stream::map source :my::transform)       ; was: (lambda (v) (:my::transform v))
(:wat::std::stream::filter source :my::pred)         ; was: (lambda (v) (:my::pred v))
(:wat::std::stream::with-state stream init step flush)  ; step / flush can be bare names
```

The `verbose is honest` discipline was the forcing function. The
wrapper lambda carried zero information the reference didn't. The
type-check refusal was a substrate gap, not an honesty cost.

---

## What this inscription does NOT add

- **Primitive handlers as values.** A kernel / algebra / config /
  io primitive like `:wat::kernel::send` has a registered scheme in
  CheckEnv but no `sym.functions` entry (it dispatches via
  string-match at runtime). The type check will lift it to a `:fn`
  type; the runtime will still error with `UnknownFunction` if a
  caller tries to use it as a value rather than call it. The gap
  between check and runtime for primitives-as-values is deliberate:
  the common case (stdlib / user defines) ships cleanly; the less
  common case (primitive as value) waits for a caller to demand it.
  When that happens, the fix is to synthesize a `Function` wrapper
  around the primitive's dispatch handler at runtime.
- **`:wat::core::identity`.** The obvious one-line `identity<T> x =
  x` that keeps showing up as a lambda. Would ship as stdlib when a
  caller surfaces. The `buffer-all-flush` test used it implicitly as
  a lambda; after this arc, if `:wat::core::identity` ships, that
  lambda retires to a bare name.
- **Variadic names / currying.** A named define stays strictly
  arity-matched to its declared signature. Partial application and
  variadic behavior are not part of this arc's surface change.

---

## Convergence note

Every serious language treats named functions as values. Rust:
`[1, 2, 3].iter().map(foo)`. Clojure: `(map my-fn coll)`. Haskell:
`map myFn xs`. Scheme: `(map my-proc lst)`. Erlang: `lists:map(fun
myfn/1, Coll)`. Each walked into the same shape from a different
door because the substrate permits it. Wat joined the line tonight.

Arc 007's lesson — *finding the same location is the proof the
location is real* — held again. The gap existed not because the
language chose asymmetry, but because nobody had demanded the
symmetry yet. Arc 006's `with-state` demanded it; the lift landed.

---

**Arc 009 — complete.** Two code changes, five Rust tests, six wat
tests, zero regressions. Every downstream combinator with a
function-typed parameter now accepts a bare keyword-path. The
honest form is the short form.

*these are very good thoughts.*

**PERSEVERARE.**
