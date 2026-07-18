# DESIGN — type-safe `query` by type name (de-mask `return-type-of`)

> **Strike (a).** Restore `(:wat::rete::query fired :weather::ColdAndWindy)` — query a fired session by
> the **type name**, checked — and kill the failure-masking the arc-294-9a flip introduced into
> `return-type-of`. Directed by the builder: *"return-type-of feels flawed… the macro just needs to
> append an `'` to the record."* This is arc 278's own no-hidden-failures law applied to `return-type-of`.

## The regression (grounded)

Pre-294-9a, `query`'s 2nd arg was a **bare type name** that *resolved to that type's constructor fn*;
`(:wat::core::defn :wat::rete::query [session ty <- :wat::core::fn] … (query-by-type-string session
(return-type-of ty)))` read the constructor's declared return type. Good UX, and **type-safe**: a typo'd
bare name is an unresolved reference → compile error.

The **294-9a flip broke it**: a bare type name is now a **kwargs macro**, not the constructor fn — so
`query fired :weather::ColdAndWindy` fails `query`'s `:wat::core::fn` param in the strict (defn-freeze)
path. Rather than fix it honestly, a **masking accommodation** was added to `eval_return_type_of`
(`runtime.rs` ~10950):

```rust
Value::wat__core__keyword(k) => { let fqdn = k.strip_prefix(':')…; return Ok(Value::String(fqdn)); }
```

It echoes **any** keyword's colon-stripped name, **unvalidated**. So a typo flows straight through:
`(query fired :weather::ColdAndWndy)` → `return-type-of :…Wndy'` → echoes `"…Wndy'"` →
`query-by-type-string` filters by a string no fact carries → **silent 0**. Confirmed by probe:
`(return-type-of :s::Nope')` on an undefined type printed `"s::Nope'"` instead of raising. **That is
failure-masking** — hiding "this type does not exist" behind a plausible string, the exact class this arc
forbids (`RVINA ERVDIT` — the ruin must educate).

Its own doc admits the motive: *"Keeps `(:wat::rete::query session :my::Type)` working with the bare type
name."* The accommodation is also only half-effective — the strict path still rejects the bare keyword at
`query`'s param, so the fixtures were forced to `query-by-type-string` anyway; only the *loose* inline
path (rune'd differentials `8a`/`8b`/`8custom`/`6b_ii_b`) rides the echo.

## Probes done (examinare — before this brief)

- **Macro mechanics ✓** — a `defmacro` takes the type-name arg (arrives as a keyword *value*), builds the
  prime `:Foo'` via `keyword/to-string` → prepend `:` → append `'` → `keyword-node` (the `core.wat:649`
  idiom), emits `(return-type-of :Foo')`; expansion resolves to `"s::Foo"`. Proven via `cargo wat`.
- **The masking flaw ✓** — the typo probe (`:s::Nope`) printed `"s::Nope'"`, no error. The disconfirming
  result that killed the "macro alone is type-safe" claim: safety needs `return-type-of` de-masked too.

## The fix (three parts)

1. **`wat/rete.wat` — `query` becomes a `defmacro`** `[session ty]` → emits
   `(:wat::rete::query-by-type-string ~session (:wat::runtime::return-type-of <ty with ' appended>))`.
   The macro appends `'` to the type keyword (the proven idiom) to reference the **prime constructor**.
   `query-by-type-string` stays the private helper + the deliberate dynamic-string escape hatch.
2. **`src/runtime.rs eval_return_type_of` — de-mask.** The `Value::wat__core__keyword` branch must
   **RAISE** on a name that is not a registered constructor/type (an honest located
   `RuntimeError`/`CheckError`: "unknown type: `:…`"), NOT echo the stripped name. A *known* type still
   resolves to its FQDN (the prime constructor's `ret_type`); only the *unknown* case changes from
   echo → raise. (`query` still returns empty for a *known-but-underived* type — that is separable and
   stays lenient; only "type does not exist" raises.)
3. **`src/check.rs:4464` — check-time validation** (for the compile-error the builder wants, not merely
   runtime): when `return-type-of`'s arg resolves to a (prime) type keyword, validate it names a
   registered type; if not → `CheckError`. So `(query fired :Typo)` fails at check time.

## Blast radius + caller flips

- `wat/rete.wat` — `query` defn → defmacro.
- `src/runtime.rs` — `eval_return_type_of` keyword branch: echo → raise-on-unknown.
- `src/check.rs` — `return-type-of` special-case: validate the type exists.
- **`tests/rete/probe_arc278_return_type_of.wat`** — currently asserts `return-type-of(:weather::ColdAndWindy)`
  (bare) — it **codifies the masking**. Move it to the constructor/prime form (`return-type-of` takes a
  fn, its documented contract), or assert the new raise-on-unknown behavior. This is a semantics change to
  a test that must be made deliberately, not silently.
- **Flip the migrated fixtures back to the front door**: `5a` (plain + with_rule), and anywhere the crusade
  used `query-by-type-string` for a *static* type → `(:wat::rete::query fired :Type)`. The rune'd inline
  callers (`8a`/`8b`/`8custom`/`6b_ii_b`) already use the bare-name form and now route through the macro.
- Existing `wat-scripts/perf/grid/*.wat` + `to-faithful-clojure-rete.wat` use `query-by-type-string`
  directly — leave (they can flip to the front door in a sweep, out of this stone's gate).

## RED gate (acceptance)

- **GREEN path**: `(:wat::rete::query fired :weather::ColdAndWindy)` type-checks and returns the derived
  facts (count matches the old `query-by-type-string` result) — in BOTH the defn-freeze and inline paths.
- **RED→caught**: `(:wat::rete::query fired :weather::ColdAndWndy)` (typo) is a **compile error** (unknown
  type), not silent 0. `(return-type-of :s::Nope')` on an undefined type **raises**, not echoes.
- **No-regression**: whole `binary_id(wat::rete)` green; `return-type-of` test updated + green; no other
  return-type-of consumer relies on the echo (grep-confirmed: only the one test + query).

## Why this is in-law, not scope-creep

`return-type-of` echoing an unknown type IS the failure-masking arc 278 exists to annihilate — the same
disease as the recv/decode `map_err(|_|)` masking (the transport-tier twin). De-masking it restores the
type-safe `query` UX **and** pulls one more masking site out by the root. The general "first-class checked
type-reference" (arc-109 `NOTE-typed-literal-constructors.md`) remains the larger stone; this strike does
not need it — the prime + a de-masked `return-type-of` suffice.
