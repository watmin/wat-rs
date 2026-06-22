# DESIGN+BRIEF — Stone: the firm doc-contract (bytes declared PERFECT)

**Status: STRIKE-READY (surface locked with the builder 2026-06-22).** This is the real 255.2
(type-sig) with 255.1b-v (@see) folded in. Supersedes `DESIGN-STONE-255.1b-v.md` (its show-source +
render-doc base is already built + uncommitted; this completes it). **Scope: prove the FULL contract
on `core::Bytes` (to-hex + from-hex).** Variadic + the 520-migration are the NEXT arc-body, NOT here.

## The contract (locked — three independent witnesses, each build-fail on divergence)
A docstring is a self-testing, self-accountable artifact. The macro extracts the *supposed* spec; we
reflect it against what's *live*; divergence fails the build.
1. Required directives present → `compile_error!` (exists).
2. `@arg` name+count ⇄ sniffed `&WatAST` params → `compile_error!` (exists).
3. **`@arg`/`@ret` TYPES ⇄ the live checker `TypeScheme`** → build-fail (NEW — the two sources are
   independent: the doc is NOT derived from the scheme; they're compared).
4. **`@example` run, result ⇄ `#=>`** → the docstrings test themselves (verify-examples, gated).
5. **Purity dictates the example kind** (NEW): pure∧det → ≥1 **runnable** `@example` MANDATORY;
   ¬(pure∧det) → ≥1 `@example-norun` MANDATORY + runnable `@example` forbidden.
6. `@see` ⇄ registry → no dangling (build-fail).
7. **Firm grammar** → one canonical form per marker; parser rejects deviations.

## Firm grammar (two classes)
**Single-line structured markers** — leading fields, `desc` = remainder; wrapped continuation =
exactly 2-space indent; **NO cosmetic separator** (the 4-way `—`/`--`/`-`/`:` DIES; render-doc adds the
` — ` at render):
```
@added   <semver>
@arg     <name> <type> <desc…>      ; type = a wat type form, e.g. :wat::core::Bytes
@ret     <type> <desc…>
@deprecated <semver> <fqdn>
@see     <fqdn>
```
**Block markers** — `@example`/`@example-norun`: everything after the marker until `#=>` is the
**form, verbatim** (multi-line OK, keeps its own indentation); after `#=>` is the expected. Inline
(`@example (f) #=> 3`) and multi-line (form on following lines, `#=>` on its own line) are the same
rule — `#=>` is the delimiter. The 2-space-wrap rule does NOT apply inside a block.

## Rooms + the build (parts; probe-first each)
**A — firm grammar (`crates/wat-doc/src/lib.rs`):** kill the 4-way separator (`:33` sep set); make
`@arg` parse `<name> <type> <desc>` and `@ret` parse `<type> <desc>` (type = the first token after
name/marker; desc = remainder); enforce 2-space-wrap for single-line markers; parse `@example`/
`-norun` as multi-line blocks split on `#=>`. A non-conforming line → `DocError` (closed enum).
**B — types onto the registry (`crates/wat-macros/src/wat_intrinsic.rs` + `src/intrinsic/mod.rs`):**
the macro carries the parsed `@arg` types + `@ret` type onto `IntrinsicSubmission`/`IntrinsicEntry`
(extend `args` to `(name, type, desc)` or add `arg_types`/`ret_type`).
**C — type cross-check (consumer-side, `src/intrinsic/` test in the `wat` crate):** walk
`registry().all_entries()`; for each, compare its doc `@arg`/`@ret` types against the checker's
registered `TypeScheme` for that FQDN (`register_builtins`/`infer_list`, `check.rs`); **mismatch →
panic/build-fail** "doc type for `<fqdn>` arg `<n>` says `<X>`, scheme says `<Y>`". The scheme is the
independent witness — do NOT derive it from the doc.
**D — purity-mandated examples (consumer-side test):** walk the registry; `pure∧det` (via
`is_effectful_op` + the nondeterministic set) → assert ≥1 example with `run=true`; else → assert ≥1
`run=false` AND no `run=true`. build-fail on violation. (Upgrades the existing one-way purity check.)
**E — @see render + kill the fake-use:** `render-doc` renders a `See also:` section from `entry.see`
(the HONEST product reader). DROP the `debug_assert!` in `registry()` (`mod.rs:254-265` — it was a
lint-appeasing fake-use, the anti-pattern `mod.rs`'s own note forbids). `check_see_refs` becomes a
`#[cfg(test)]` helper (validation only). Add `@see` cross-refs to bytes: to-hex ↦ from-hex + back.
**F — re-fit bytes (`src/intrinsic/bytes.rs`):** rewrite both doc-comments to the firm grammar
(types in `@arg`/`@ret`, `@see` refs). bytes is pure∧det → its `@example`s stay runnable (rule 5).

## RED probes (verify RED before each part)
- **render @see:** `wat-tests/reflect/reflection-surface.wat` already asserts `render-doc` contains
  `from-hex` — RED now (no See-also). GREEN after E+F.
- **type cross-check:** a test that flipping bytes' `@arg` type to a wrong type (e.g. `:wat::core::String`)
  makes the cross-check FAIL; correct type passes. (Author it, verify it bites, restore.)
- **firm grammar:** a wat-doc unit test that the canonical `@arg name type desc` parses AND a
  4-way-separator line is now REJECTED (was accepted).

## STOP triggers
1. If the checker's `TypeScheme` for bytes isn't cleanly readable from a consumer test (the
   register_builtins/infer_list shape isn't queryable per-FQDN), STOP and report — the cross-check
   needs that read; don't fake it.
2. If re-fitting `@arg <name> <type>` collides with the existing name+count mutual-check (which only
   knew name+count), STOP and report — the check must extend to accept the type token, not break.
3. Variadic is OUT of scope — if bytes somehow needs it, STOP (it doesn't; both handlers are 1-arg).

## Gate — "bytes is PERFECT"
- reflection-surface.wat green (show-source + render-doc incl. See-also/from-hex).
- type cross-check green (and bites when a type is wrong); purity-mandated-examples green.
- @see-check green; the `debug_assert` is GONE; no dead-code; no `#[expect(dead_code)]` fake-use.
- bytes' two docs conform to the firm grammar; 4-way separator now rejected by the parser.
- lib floor (958+new/36/1); wat-tests floor (268+new/1); clippy clean.
- **Every contract axis (1-7) holds on bytes, by an independent witness.** That is "perfect."

## Out of scope (named — the NEXT arc-body)
Variadic `@arg xs… <elem-type>` + macro `&[WatAST]` support (arrives with the first variadic
intrinsic in the migration); the ~520-intrinsic migration; the fuzzy-docs MCP (HORIZON note).
