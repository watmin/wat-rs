# wat-rs arc 054 — Idempotent re-declaration — INSCRIPTION

**Status:** shipped 2026-04-25. Ninth wat-rs arc post-known-good.

A small, narrowly-scoped relaxation of the registry rules: re-
declaring a name with **byte-equivalent body** is a no-op;
re-declaring with **divergent body** still errors. Three registries
gained the rule (typealias, define, defmacro). Behavior change is
strictly monotone-relaxing — anything that errored before with
divergent bodies still errors; anything that errored with identical
bodies (always a false positive) now succeeds.

Builder direction:

> "we need to work on this next
> wat-rs/docs/arc/2026/04/054-idempotent-redeclare/"

The lab's CandleStream shim (the first in-crate shim) ships its wat
surface via `wat_sources()` AND as an on-disk file loaded by
`main.wat` and test preludes. Both paths registered the same
typealias, hitting:

```
duplicate type declaration: :lab::candles::Stream
```

This arc closes that papercut. The lab and any future in-crate shim
gets the property for free.

---

## What shipped

### Slice 1 — Typealias idempotency

`src/types.rs` `register` and `register_stdlib`:

```rust
if let Some(existing) = self.types.get(&name) {
    if existing == &def {
        return Ok(());
    }
    return Err(TypeError::DuplicateType { name });
}
```

`TypeDef` derives `PartialEq`; `WatAST::PartialEq` is span-agnostic
because `Span::PartialEq` always returns true (verified via inspection).
No normalization needed.

### Slice 2 — Define idempotency

`src/runtime.rs` `register_defines` and `register_stdlib_defines`:
same shape. The body comparison is structural-equality on the
function's params, type_params, param_types, ret_type, and body
AST. New helper:

```rust
fn function_byte_equivalent(a: &Function, b: &Function) -> bool {
    a.params == b.params
        && a.type_params == b.type_params
        && a.param_types == b.param_types
        && a.ret_type == b.ret_type
        && *a.body == *b.body
}
```

### Slice 3 — Defmacro idempotency

`src/macros.rs` `MacroRegistry::register` and `register_stdlib`:
same shape. The body comparison is on params, rest_param, and body
AST. New helper:

```rust
fn macro_byte_equivalent(a: &MacroDef, b: &MacroDef) -> bool {
    a.params == b.params && a.rest_param == b.rest_param && a.body == b.body
}
```

The pre-existing `duplicate_defmacro_rejected` lib test was
rewritten — under the old behavior, byte-equivalent re-registration
errored, so the test re-registered an identical macro. Under the
new behavior, that's a no-op. The test was renamed to
`duplicate_defmacro_with_divergent_body_rejected` (template body
changed) and a new `duplicate_defmacro_byte_equivalent_is_noop`
lib test was added covering the relaxed path.

### Slice 4 — Diagnostic on the still-error path *(deferred)*

The DESIGN/BACKLOG flagged a follow-up that extends the bare
`duplicate type declaration: :X` error to include both source
spans for the divergent case. Not shipped in this arc — the
behavior change (slices 1–3) was the immediate ergonomic blocker.
Slice 4 stays in the BACKLOG as ready-to-ship; can land in or out
of band as a "054.5" arc when an author trips on the diagnostic
again.

### Slice 5 — Docs

This INSCRIPTION + USER-GUIDE addendum + lab FOUNDATION-CHANGELOG
row.

---

## Tests

`tests/wat_idempotent_redeclare.rs` — 6 integration tests:

1. `typealias_byte_equivalent_is_noop` — two identical typealias
   forms, freeze succeeds.
2. `typealias_divergent_errors` — `:my::Amount :f64` then
   `:my::Amount :i64`; freeze fails with duplicate-type error.
3. `define_byte_equivalent_is_noop` — two identical
   `(define :my::add-one ...)` forms, freeze succeeds.
4. `define_divergent_body_errors` — same signature, body changed
   from `(+ a 1)` to `(+ a 2)`; freeze fails.
5. `defmacro_byte_equivalent_is_noop` — two identical
   `(defmacro :my::ident ...)` forms.
6. `shim_double_register_pattern_works` — the motivating shape:
   the same `(typealias :lab::candles::Stream :i64)` reaches the
   registry twice (simulating wat_sources() + on-disk file
   delivery). Freeze succeeds.

Plus 1 new lib test (`duplicate_defmacro_byte_equivalent_is_noop`)
and 1 renamed lib test (`duplicate_defmacro_rejected` →
`duplicate_defmacro_with_divergent_body_rejected`).

---

## Architecture decisions resolved

### Q1 — What counts as "byte-equivalent"?

Structural equality on the parsed AST, post-macro-expansion.
`WatAST::PartialEq` is span-agnostic (Span's `PartialEq` always
returns true), so two ASTs parsed from different source paths but
carrying identical content compare equal. No normalization layer
needed.

### Q2 — Performance

Equality comparison fires once per registration; registrations
happen at startup. A typical define body is ~20 nodes, comparison
is microseconds. Negligible.

### Q3 — Generalize beyond the three registries?

Not in this arc. Surveying others:
- `(:wat::core::use! ...)` — already idempotent.
- `enum` / `struct` declarations — equality semantics need design;
  defer to a follow-up arc when a real conflict shows up.
- `(:wat::config::set-*!)` — already has "first-call wins / set
  once" rule from arc 045; not a redeclaration concern.

The three forms covered are the ones the in-crate-shim path
actually hits. Other forms can adopt the same rule when a caller
cites a use case.

### Q4 — User code that *intentionally* re-declares to override?

There is no such pattern in wat today. Shadowing is via lexical
scope (`let*`, sandbox). If a "redefine!" form is ever introduced
for REPL workflows, this arc's idempotency becomes the floor and
the redefine form opts out via a different code path. No conflict.

---

## Count

- Sites changed: **3** registries (types.rs, runtime.rs, macros.rs)
  × **2** functions each (register + register_stdlib) = **6**
  insertion paths.
- New helpers: **2** (`function_byte_equivalent`,
  `macro_byte_equivalent`). Type equality reuses derived
  `PartialEq` on `TypeDef`.
- New integration test crate: **1** (`tests/wat_idempotent_redeclare.rs`)
  with **6** cases.
- Lib tests: **+1** new (byte-equivalent defmacro), **+0** net
  (one renamed) = **611 → 612**.
- Clippy: **0** warnings.

---

## Reckoner labels signature shift (in-flight, not part of this arc)

While slice 1–3 was in flight, holon-rs landed
`ReckConfig::Discrete(Vec<HolonAST>)` — labels became arbitrary
HolonAST values (per the user's "labels /are/ ASTs — of arbitrary
complexity" direction). The wat-side surface for
`:wat::holon::Reckoner/new-discrete` followed:
`Vec<String>` → `Vec<wat::holon::HolonAST>`. Existing
arc-053 reckoner tests updated to construct labels via
`(:wat::holon::Atom "up")`. Recorded here for chronology; no
separate arc.

---

## What this unblocks

- **In-crate shim ergonomics.** Lab's CandleStream is the first;
  future shims (sqlite ledger, websocket OHLCV source, NATS sink,
  redis sink) get the property for free.
- **Library re-export.** A wat module that re-exports a typealias
  from an upstream dep no longer risks duplicate-declaration
  errors when both consumers happen to be active.
- **Test prelude flexibility.** `make-deftest` preludes can load
  files that are also delivered via `wat_sources()` without
  staging concerns.
- **REPL / hot-reload (passive enabler).** Future tooling that
  re-loads a file mid-session no longer fights the registry.

---

## Follow-through

- **Slice 4** (diagnostic improvement on divergent path) stays in
  the BACKLOG. Ships when an author's troubleshooting time spent
  hunting "where did the other declaration come from" justifies
  the half-day work.
- **Other forms** (enum / struct redeclaration) — adopt the same
  rule when a caller materializes the use case.

---

## Commits

- `<wat-rs>` slices 1 + 2 + 3 + tests + INSCRIPTION (this commit).
- `<lab>` FOUNDATION-CHANGELOG row.

---

*"Identical content is identical content; the error is reserved for
conflicting content."*

**PERSEVERARE.**
