# wat-rs arc 054 — Idempotent re-declaration for typealias / define / defmacro

**Status:** opened 2026-04-25. Ninth wat-rs arc post-known-good.

**Scope:** small. Three registries gain an "if byte-equivalent, no-op"
rule. Behavior change strictly relaxes existing errors.

Builder direction:

> "wat-rs changes are getting rarer now - we need to address them
> when they show up...."

> "we may have encountered a problem that needs wat-rs to deal with
> for us..."

> "i'm almost at a pause point in my current wat-rs work - can you
> identify the work that's necessary in wat-rs to make this
> ergonomic"

---

## Motivation

In-crate shims are now a real shape — holon-lab-trading is shipping
its first one (parquet OHLCV reader → `:rust::lab::CandleStream`,
2026-04-25). The shim's wat surface (`wat/io/CandleStream.wat`)
naturally lives under the lab's own `wat/` tree, alongside every
other lab wat file. That's the project's convention; deviating from
it for shim-supplied surfaces would be jarring.

The problem: the shim *also* declares the file in its
`wat_sources()` (so it's discoverable as a dep contribution), and
arc 015's `installed_dep_sources()` is **eagerly evaluated** at
every freeze (`wat-rs/src/stdlib.rs:112`). The on-disk file is
*also* loaded — by `wat/main.wat`'s `(:wat::load-file!)` for the
binary, by test preludes via `make-deftest` for tests. Two paths to
the same source. Both run. Result:

```
duplicate type declaration: :lab::candles::Stream
```

This errors today even though both registrations carry **byte-
identical** bodies — they came from the same `WatSource` via two
delivery channels.

External crates (wat-lru, future siblings) don't hit this because
their wat surface lives only inside the crate's source tree; the
consumer's filesystem has no copy. The conflict is a property of
in-crate shims specifically, where the wat surface naturally
double-exposes.

**Today's options for the lab are both unergonomic:**

1. **Disk-only.** Shim returns `&[]` from `wat_sources()`. The
   on-disk file is loaded explicitly by `main.wat` and every test
   prelude. Means every test that uses the shim writes
   `(:wat::test::make-deftest :deftest ((:wat::load-file! ...)))`
   even though the shim is the natural carrier.
2. **Bake-only.** Shim's `include_str!` points at a file outside
   `wat/` (e.g. `src/wat_baked/CandleStream.wat`). Lab's wat
   surface gets split: most files in `wat/`, shim files in
   `src/wat_baked/`. Two locations for "the wat tree" is harder
   to reason about than one.

Neither is awful. Both are taxes that compound as more in-crate
shims ship. The lab's roadmap has at least three more (a sqlite
ledger, a websocket source, a NATS/redis sink) — the tax recurs.

**The principle being violated.** Re-running a definition with
identical body should be a no-op, not an error. Identical content
is identical content; the error is reserved for *conflicting*
content. Today's check is too coarse.

---

## What ships

A single principle applied to three registries:

> Re-registering a name with byte-equivalent body is a no-op.
> Re-registering with differing body remains an error.

Concretely:

### 1. Typealias registry (`check.rs` / wherever
`(:wat::core::typealias :X :Y)` lands)

**Before.**
```rust
if registry.contains(name) {
    return Err(DuplicateTypeAlias { name });
}
registry.insert(name, target);
```

**After.**
```rust
if let Some(existing) = registry.get(name) {
    if existing == &target { return Ok(()); }   // no-op
    return Err(DuplicateTypeAlias { name, existing: existing.clone(), new: target });
}
registry.insert(name, target);
```

### 2. Define registry (`runtime::register_defines`)

Same shape. The "body" comparison is the function's parameter list
+ return type + body AST. Two `define` forms with byte-identical
shape and body re-register without error.

### 3. Macro registry (`macros::register_defmacros`)

Same shape. The "body" is the macro's pattern + template AST.

---

## Decisions resolved

### Q1 — What counts as "byte-equivalent"?

**Structural equality on the parsed AST**, post-macro-expansion.
Whitespace and comment differences are normalized away by the
parser; what reaches the registry is the canonical form.

`WatAST` already implements `PartialEq` (used by other equality
sites in the runtime). Reuse it. No serialization round-trip; no
hashing tricks; just `==`.

Edge case: spans. Two ASTs with identical content but different
source spans (e.g., one came from `wat-lab/io/CandleStream.wat`
in `wat_sources()`, the other from `wat/io/CandleStream.wat`
on-disk) should compare equal. `WatAST::PartialEq` should already
ignore spans (verify); if it doesn't, normalize before compare.

### Q2 — What about user code that *intentionally* re-declares
to override?

There is no such pattern in wat today. `(:wat::core::define ...)`
is "declare once"; shadowing is via lexical scope (`let*`,
sandbox), not redefinition. `(:wat::core::typealias)` similarly.
`(:wat::core::defmacro)` likewise.

If the rule changes in the future — e.g., a "redefine!" form is
introduced for REPL-style workflows — this arc's idempotency
becomes the floor, and the re-define form opts out via a
different code path. Doesn't conflict.

### Q3 — Performance cost

Equality comparison on `WatAST` is structural recursion. For a
typical define body (~20 nodes), this is microseconds. The check
fires once per registration; registrations happen at startup,
not on hot paths. Negligible.

### Q4 — Diagnostic on the *other* error path?

When the redeclaration *does* differ — the rare-but-real case
where two different bodies want the same name — the error today
is one-line: `duplicate type declaration: :lab::candles::Stream`.
No source location for either site.

This arc's primary slice keeps that diagnostic as-is — the
behavior change is the no-op rule. A follow-up slice (slice 4 in
BACKLOG) extends the error to include both source spans:

```
duplicate type declaration: :lab::candles::Stream
  first declared:  installed_dep_sources()/io/CandleStream.wat:8:3
  second declared: wat/io/CandleStream.wat:8:3
```

Slice 4 is independent of slices 1-3 and could ship separately.

### Q5 — Should this generalize beyond typealias/define/defmacro?

Surveying other registries:

- `(:wat::core::use! ...)` — already idempotent (per inspection).
- `enum` declarations (058-048) — equality semantics for enum
  bodies need design; defer to a follow-up if a real conflict
  shows up.
- `struct` declarations — same as enum; defer.
- `(:wat::config::set-*!)` — already has a "first-call wins / set
  once" rule per arc 045; not a redeclaration concern.

This arc covers the three forms that actually fire on the
in-crate-shim path. Other forms can adopt the same rule when a
caller cites a use case.

---

## Implementation sketch

```
src/check.rs (or wherever typealias is registered):
  - find the duplicate-error site for typealias
  - wrap in: if existing == &new { return Ok(()); }
  - add test asserting byte-identical → ok, differing → err

src/runtime.rs (register_defines):
  - same shape for `define`
  - careful: registration is the WatAST function body, not a
    compiled closure. Compare ASTs.

src/macros.rs (register_defmacros):
  - same shape for `defmacro`
  - compare pattern + template AST
```

Each of the three changes is ~10-15 LOC plus a passing test and
a re-passing-after-change negative test.

---

## Tests

`tests/wat_idempotent_redeclare.rs` — five cases, runtime-end-to-
end:

1. **Typealias byte-equivalent — no-op.**
   Parse and freeze a wat program that contains
   `(:wat::core::typealias :X :Y)` twice. Asserts no error.

2. **Typealias byte-different — errors with diagnostic.**
   `(:wat::core::typealias :X :Y)` then
   `(:wat::core::typealias :X :Z)`. Asserts the error fires and
   carries the existing target.

3. **Define byte-equivalent — no-op.**
   Two `(:wat::core::define (:foo (a :i64) -> :i64) (:wat::core::+ a 1))`
   forms in the same program. No error.

4. **Define byte-different — errors.**
   Different body for the same `:foo` signature. Errors with
   diagnostic.

5. **Defmacro byte-equivalent — no-op.**
   Two identical defmacro registrations. No error.

Plus an integration test that exercises the original in-crate-
shim shape: a `mod shims` that contributes a `wat_sources()` pointing
at an on-disk-loaded path. Both registrations succeed; the
typealias resolves; a wat program using the type compiles and runs.

---

## Sub-fogs

### 5a — Span equality

Verify `WatAST::PartialEq` ignores `SourceSpan`. If it doesn't,
the byte-equivalent rule will mis-fire (two AST trees parsed
from "different files" would always compare unequal even with
identical content).

Inspection of `parser.rs` should confirm. If spans participate
in `PartialEq`, normalize via a `WatAST::without_spans()` helper
before compare.

### 5b — Re-export interaction

If a wat program imports a typealias from a dep AND from another
dep that re-exports it (chain of `(:wat::core::use! ...)` +
`(:wat::core::typealias ...)`), this is structurally a re-
declaration. After this arc, that pattern just works. Verify in
slice 1's test.

### 5c — Order sensitivity

Both registrations resolve to the same final state regardless of
order: first-then-second is a register-then-no-op; second-then-
first is a register-then-no-op (same outcome). Confirm in tests.

---

## What this arc does NOT add

- **A "redefine!" form for runtime mutation.** Out of scope.
- **REPL-style "edit and re-load" workflow.** This arc enables it
  *passively* (re-loading the same file twice now works) but adds
  no new mechanisms.
- **Idempotency for enum / struct declarations.** Defer to a
  follow-up arc once a use case shows up. The three forms
  covered are the ones the in-crate-shim path actually hits.
- **Loader-level dedup.** A different fix would be: detect that
  `(:wat::load-file! "P")` resolves to source already in
  `installed_dep_sources()` and skip. That is a bigger change
  (loader becomes dep-aware), and is unnecessary if registration
  is idempotent. Rejected as redundant.
- **Behavior change for *conflicting* re-declarations.** Those
  still error. The relaxation only covers byte-identical cases.
- **Performance optimization for the deps mechanism's eager
  load.** Out of scope; the auto-load is correct as designed.

---

## Non-goals

- **Lab-side adoption beyond CandleStream.** The lab's first
  in-crate shim provoked this arc. Subsequent shims (sqlite
  ledger, websocket source, NATS sink) inherit the property
  without further work. No proactive rewrite of existing
  patterns required.
- **Error-message overhaul.** Slice 4 is opt-in, not a blocker
  for slices 1-3. The bare diagnostic improvement could ship
  in a separate small arc if this one closes without it.
- **058-NNN INSCRIPTION addendum.** This arc is a runtime
  semantics fix, not a language-surface addition. No 058 sub-
  proposal needed; the substrate's grammar is unchanged.
  FOUNDATION-CHANGELOG entry only.
- **USER-GUIDE rewrite.** A short note in the in-crate-shim
  section ("idempotent redeclaration means you can ship the
  same wat file via on-disk and `wat_sources()` without
  conflict") suffices.

---

## What this unblocks

- **In-crate shim ergonomics.** Lab's CandleStream is the first.
  Future shims (sqlite ledger, websocket OHLCV source, NATS
  sink, redis sink) get the same property for free.
- **Library re-export.** A wat module that re-exports a
  typealias from an upstream dep no longer risks duplicate-
  declaration errors when both consumers happen to be active.
- **Test prelude flexibility.** `make-deftest` preludes can load
  files that are also delivered via `wat_sources()` without
  staging concerns. Authors stop having to track which path
  delivered which form.
- **REPL / hot-reload (passive enabler).** Future tooling that
  re-loads a file mid-session no longer fights the registry.
