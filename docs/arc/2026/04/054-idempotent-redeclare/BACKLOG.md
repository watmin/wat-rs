# wat-rs arc 054 — idempotent re-declaration — BACKLOG

**Shape:** four slices. Three registries gain idempotency
independently; a fourth optional slice extends the diagnostic
on the still-error path. Each slice is independently mergeable
and independently testable.

---

## Slice 1 — Typealias idempotency

**Status: ready.**

`src/check.rs` (or wherever `(:wat::core::typealias :X :Y)`
hits the type registry):

- Find the duplicate-declaration error site for typealias.
- Wrap the insert: `if existing == &new { return Ok(()); }`
  before raising the error.
- Verify `WatAST::PartialEq` (or whatever the alias-target type
  is) ignores `SourceSpan`. If not, normalize before compare.

`tests/wat_idempotent_redeclare.rs` — first two cases:

1. **Byte-equivalent typealias — no error.**
   Parse a wat program with `(typealias :X :Y)` twice. Freeze
   succeeds. Type still resolves to `:Y`.

2. **Byte-different typealias — error preserved.**
   `(typealias :X :Y)` then `(typealias :X :Z)`. Freeze fails
   with the existing diagnostic.

**Estimated cost:** ~15 LOC + 2 tests. Half a day.

**Unblocks:** the lab's CandleStream smoke test (the immediate
caller).

---

## Slice 2 — Define idempotency

**Status: ready (after slice 1, for shared test infra).**

`src/runtime.rs` `register_defines` (or its equivalent post-arc
sweep):

- Same shape as slice 1. Compare the AST of the function body +
  param list + return type.
- Add: `if existing.body == new.body && existing.params == new.params && existing.ret == new.ret { return Ok(()); }`.

Tests:

3. **Byte-equivalent define — no error.**
   Two `(define (:foo (a :i64) -> :i64) (:+ a 1))` forms in the
   same program. Freeze succeeds. Calling `:foo` returns the
   expected value.

4. **Byte-different define — error.**
   Different body for the same name. Errors.

**Estimated cost:** ~15 LOC + 2 tests. Half a day.

**Unblocks:** wat library files that ship the same `define` via
both an on-disk path and a dep contribution.

---

## Slice 3 — Defmacro idempotency

**Status: ready (after slice 2, for shared test infra).**

`src/macros.rs` `register_defmacros`:

- Same shape. Compare pattern + template AST.

Tests:

5. **Byte-equivalent defmacro — no error.**
   Two identical `(defmacro :M ...)` forms. Freeze succeeds.
   Macro expands as expected.

(Slice 3 doesn't need a "byte-different" companion test —
slices 1 and 2 cover the divergence path; macro just
inherits the same pattern.)

**Estimated cost:** ~15 LOC + 1 test. Half a day.

---

## Slice 4 — Diagnostic improvement on the still-error path *(optional)*

**Status: ready, can ship in or out of band.**

When the redeclaration *is* divergent (slices 1-3's else-branch),
the current diagnostic is bare:

```
duplicate type declaration: :lab::candles::Stream
```

Extend each registry's error to carry both source spans:

```
duplicate type declaration: :lab::candles::Stream
  first declared:  installed_dep_sources()/io/CandleStream.wat:8:3
  second declared: wat/io/CandleStream.wat:8:3
```

This requires the registry to retain the first registration's
span (it likely already does for other diagnostics; verify). If
not, plumb a span through.

**Estimated cost:** ~30 LOC + 1 test (assert error contains both
spans). Half a day.

**Independent of slices 1-3.** Could ship as a standalone arc
("054.5 — duplicate-declaration diagnostic") if 054 closes
without it. Captures a real ergonomics improvement (the day-
in-the-life of the in-crate-shim work blew an hour because the
error didn't say where the duplicate came from).

---

## Slice 5 — INSCRIPTION + USER-GUIDE addendum

**Status: blocked on slices 1-3 shipping.**

- **INSCRIPTION.md** — record the three registry sites, the
  PartialEq verification result, the test count, the LOC delta.
  Standard inscription shape.
- **USER-GUIDE addendum.** Short addition to §"Add a
  `src/shim.rs` module" (the in-crate-shim section): a paragraph
  noting that the shim's wat surface can live on-disk *and* in
  `wat_sources()`, with idempotent redeclaration handling the
  collision. Mentions that wat-lru's pattern (bake-only, no
  on-disk version) and the lab's pattern (disk + bake) are both
  valid.
- **FOUNDATION-CHANGELOG row.** A line documenting the rule:
  "registration of byte-equivalent re-declarations is a no-op;
  divergent re-declarations remain an error."

**Estimated cost:** ~1 hour. No code changes; doc only.

---

## Verification end-to-end

After all slices land, the lab's CandleStream shim can be
expressed in either of these shapes without error:

```rust
// Shape A: bake-only (wat-lru style)
pub fn wat_sources() -> &'static [WatSource] {
    static FILES: &[WatSource] = &[WatSource {
        path: "io/CandleStream.wat",
        source: include_str!("../wat/io/CandleStream.wat"),
    }];
    FILES
}
// (with wat/io/CandleStream.wat ALSO on disk, loaded by main.wat
//  and test preludes — was failing before this arc, passes after)
```

```rust
// Shape B: disk-only (current workaround)
pub fn wat_sources() -> &'static [WatSource] { &[] }
```

Both compile, both run, both make `:lab::candles::*` available
to consumers. The author picks based on whether they want the
file auto-loaded by deps composition or explicitly via
`(:wat::load-file!)`. No more "two paths to the same source =
duplicate registration" papercut.

---

## Out of scope

- **Loader-level dedup** ("if `load-file P` resolves to source
  already in `installed_dep_sources()`, skip"). Bigger change
  (loader becomes dep-aware); unnecessary once registration is
  idempotent. Rejected.
- **Mutable re-declaration.** No "(redefine! ...)" form. This arc
  is a relaxation of the error rule, not a new mechanism.
- **Idempotency for `enum` / `struct` declarations.** The three
  forms covered (typealias, define, defmacro) hit the in-crate
  shim path. Other forms can adopt the same rule when called
  for. Defer.
- **Performance optimization** of the eager-load behavior at
  `stdlib.rs:112`. The auto-load is correct; the bug is the
  registration's intolerance, not the auto-load's frequency.

---

## Risks

**None identified.** The behavior change is monotone-relaxing: anything
that errored with byte-different bodies still errors; anything that
errored with byte-identical bodies (which was always a false-positive)
now succeeds.

The only risk vector is `WatAST::PartialEq` not being span-agnostic
— sub-fog 5a in DESIGN. That's verified at slice 1 implementation
time; if it fails, normalize-before-compare is a 5-line addition.

---

## Total estimate

- Slice 1: half day
- Slice 2: half day
- Slice 3: half day
- Slice 4 (optional): half day
- Slice 5: 1 hour

**Two days end-to-end** if slice 4 ships in-arc; one and a half
without.
