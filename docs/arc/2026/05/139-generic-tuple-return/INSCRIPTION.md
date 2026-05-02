# Arc 139 — INSCRIPTION

## Status

**Shipped + closed 2026-05-03.** One slice + docs.

- **Slice 1** — `canonical_callable_name` helper + 3 lookup-site
  patches across `src/runtime.rs`, `src/resolve.rs`, `src/check.rs`.
  Probe restored from `:should-panic` to passing deftest. Commit
  `28b185f`.
- **Slice 2** — this INSCRIPTION + 058 row.

The arc surfaced when arc 135 slice 3's sonnet sweep hit
`unknown function: :test::make-3tuple<wat::core::bool>` and
mis-attributed the failure as a generic-T return-type bug. Arc 138's
infrastructure (errors carry coordinates) plus a directly-isolated
probe (`tmp-3tuple-probe.wat`) corrected the diagnosis: it was a
substrate registration/lookup asymmetry, not a tuple-return bug.

## What this arc fixes

The substrate registers user-defined generics under their canonical
name (sans turbofish). `parse_define_form` in `src/runtime.rs`
calls `split_name_and_type_params(:my::helper<T>)` → registers as
`:my::helper`. Symmetric strip at registration time.

Pre-arc-139, NO call-site lookup did the symmetric strip. A user
write `(:my::helper<wat::core::bool> arg)` invoked literal-string
lookup of `:my::helper<wat::core::bool>` — not registered; runtime
fired `UnknownFunction`. Three lookup sites had this gap:

- `src/runtime.rs::eval_call`'s user-define dispatch arm.
- `src/resolve.rs::is_resolvable_call_head`'s `sym.get(head)` check.
- `src/check.rs::infer_list`'s `env.get(k)` for the call's TypeScheme.

After arc 139, all three strip via `canonical_callable_name(kw)`
before lookup. Symmetric registration vs lookup; the substrate
honors its own polymorphic-define contract.

## The helper

`pub fn canonical_callable_name(kw: &str) -> &str` in
`src/runtime.rs`. **Balanced-suffix rule**: only strips when the
keyword ends in `>` AND contains a `<`. Comparison operators like
`:wat::core::f64::<` end with `<` and have no closing `>` — those
are NOT turbofish suffixes; they pass through unchanged. The lexer
admits both forms (depth tracking permits unmatched trailing `<`);
arc 139's strip distinguishes them by the closing `>` invariant.

## Tests

- `wat-tests/tmp-3tuple-probe.wat` — turbofish call form
  `(:test::make-3tuple<wat::core::bool> true)` from a deftest's
  prelude-defined `:test::make-3tuple<T>`. Passes.
- `wat-tests/tmp-3tuple-inferred.wat` — inferred-T call form
  `(:test::make-3tuple true)` (no turbofish). Already passed
  pre-arc-139; still passes.

`cargo test --release --workspace`: 0 failures across wat-rs.
Trading lab tests are intentionally broken (out of scope until
wat-rs settles; user direction 2026-05-03).

## Generalizes

Arc 102 (`eval-ast!` polymorphic return) is the closest structural
precedent: substrate primitive's polymorphic scheme didn't match
runtime; arc 102 aligned them. Same shape here: substrate's
symmetric registration/lookup contract was half-built; arc 139
closes the missing half.

The two arcs share a discipline: **the substrate's own contracts
must be symmetric**. If registration strips, lookup strips. If a
scheme claims polymorphism, runtime honors polymorphism. When the
substrate is internally inconsistent about a contract, users trip
over the gap and the substrate must close it.

## Limitations

- Generic-T type ARGUMENTS at the call site (`<wat::core::bool>`)
  are currently discarded by the lookup path. The type checker
  uses its own instantiation machinery (free fresh vars) to bind
  T from arg types via unification — same as before arc 139. The
  turbofish is ergonomic only at the moment; a future arc could
  use the explicit type args to constrain inference at the call
  site (turbofish as a type-annotation-equivalent for generics).

## Cross-references

- DESIGN: `docs/arc/2026/05/139-generic-tuple-return/DESIGN.md`.
- `src/runtime.rs::canonical_callable_name` — the helper.
- `src/runtime.rs::split_name_and_type_params` — the symmetric
  registration-side strip (predates arc 139).
- `docs/arc/2026/04/102-eval-ast-polymorphic-return/INSCRIPTION.md`
  — sibling polymorphism-alignment arc.
- `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the arc that
  enables the diagnosis path; without spans on errors, the bug
  would have been mis-attributed as a tuple-return issue (and was
  initially).

## What this arc closes

The substrate's "user-define generics" surface is now coherent
end-to-end. Define `:my::helper<T>` → register canonical → invoke
turbofish `<wat::core::bool>` OR inferred → both resolve to the
same registered function. The asymmetry that burned sonnet's slice
3 (and would have burned every future user trying turbofish) is
sealed.
