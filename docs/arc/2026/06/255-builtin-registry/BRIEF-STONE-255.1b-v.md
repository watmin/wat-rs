# BRIEF — Stone 255.1b-v: reflection surface (show-source + render-doc + @see-check)

Read `DESIGN-STONE-255.1b-v.md` first (the surface is settled: render-doc/show-source
RETURN Strings; the caller prints; @see-check is a consumer test).

## The work
Add the reflection surface over the intrinsic registry, proven on the `core::Bytes` pilot:
two pure verbs (`show-source`, `render-doc`) returning Strings, plus the `@see` registry-check.
GREEN gate: `wat-tests/reflect/reflection-surface.wat` (on disk, RED-verified).

## Ground the copy-from pattern FIRST
`:wat::runtime::metadata-of` (`src/runtime.rs:3984` → `eval_metadata_of`, ~runtime.rs:10119+) is
your template: it takes a single FQDN-keyword arg, looks the intrinsic up via
`crate::intrinsic::registry().lookup_entry(name)` (runtime.rs:10149), and reads the entry's
fields. show-source + render-doc take the SAME arg shape and reading. READ metadata-of end to
end (arg extraction + the checker side) before writing the two verbs — copy its arg-typing.

## Part A — show-source
1. `crates/wat-macros/src/wat_intrinsic.rs` (the `emit` fn ~209-290, where `args_lit`/
   `examples_lit` are built into the `IntrinsicSubmission` `quote!`): capture the handler source
   via `quote!(#item).to_string()` (stable restringify; `proc_macro::Span::source_text` is
   nightly — do NOT use it) and emit a `source: <lit>` field on the submission.
2. `src/intrinsic/mod.rs`: add `source: &'static str` to BOTH `IntrinsicSubmission` (~:140) and
   `IntrinsicEntry` (~:150); carry it in `registry()` (~:225).
3. `src/intrinsic/reflect.rs`: a `#[wat_intrinsic(":wat::core::show-source")]` handler, 1 arg
   (the FQDN keyword), `-> :wat::core::String`. Dispatch: `registry().lookup_entry(fqdn)` →
   `.source`; ELSE user form via `sym` → its AST → `(:wat::core::write-forms …)`
   (`eval_write_forms`, `edn_shim.rs:279`). Doc it to the `#[wat_intrinsic]` contract (prose +
   @added + @arg + @ret + @example-norun — show-source's output isn't doctestable, use -norun).

## Part B — render-doc
- `src/intrinsic/reflect.rs`: a `#[wat_intrinsic(":wat::core::render-doc")]` handler, 1 arg (FQDN
  keyword), `-> :wat::core::String`. Read the same entry (or call the metadata-of machinery) and
  FORMAT a plain-text block with newlines: a name/signature line, the `prose`, then `Examples:`
  with each example's `expr` (+ ` #=> <expected>` where present). Pure/deterministic → @example
  (it returns a String, assertable). Plain-text ONLY (no flavor arg — glow is a later strike).

## Part C — @see registry-check
- A consumer-side test (`tests/` Rust OR a wat-tests harness) walking
  `crate::intrinsic::registry().all_entries()`: for each `entry.see` FQDN assert it resolves to a
  registered intrinsic (`registry().lookup(fqdn).is_some()`) or a resolvable user form — fail loud
  "dangling @see `<fqdn>` on `<owner>`". This RETIRES the `see` field's `#[expect(dead_code)]` on
  `IntrinsicEntry` (`mod.rs`) — remove the attribute; the reader has landed.

## Checker (the pre-255.2 reality)
Until 255.2 moves type-sig onto the registry, new intrinsics still need a checker type entry like
`core::Bytes` has (`src/check.rs:17187-17201`). Add `infer_list`/registration entries for
`show-source` + `render-doc` (arg: the FQDN-keyword shape metadata-of uses; ret: `:wat::core::String`).
Copy metadata-of's checker handling for the keyword arg. **STOP-1:** if metadata-of's arg-typing
isn't a clean copy for these, STOP and report — don't guess the keyword-arg type.

## STOP triggers
1. **STOP-1** (above): keyword-arg typing not obviously copyable from metadata-of → stop+report.
2. **STOP-2:** if `quote!(#item).to_string()` doesn't compile in the macro (the `ItemFn` isn't in
   scope at the emit site), STOP — report what `emit` receives; do NOT reach for nightly source_text.
3. **STOP-3:** if removing the `see` `#[expect(dead_code)]` triggers a *different* dead-code
   warning (another unread field), STOP and list it — don't silence with `#[allow]`.

## Blast radius
`crates/wat-macros/src/wat_intrinsic.rs`, `src/intrinsic/mod.rs`, `src/intrinsic/reflect.rs`,
`src/check.rs` (2 type entries), one new test file. No runtime.rs dispatch arms (the verbs are
`#[wat_intrinsic]`, routed via the registry branch already at runtime.rs:4574).

## Verify (run yourself, REAL results)
After `touch tests/test.rs`:
- `cargo test --release -p wat --test test reflect` → both reflection-surface tests PASS.
- `cargo test --release` the @see-check test → green (no dangling @see in the corpus).
- `cargo test --release --lib` → floor 957/36/1 + any new (no regressions).
- `cargo test --release -p wat --test test` → floor (266 + the 2 new reflect tests = 268); only `test-run-string-entry-direct` fails.
- `cargo clippy --release` → no new warnings; the `see` dead-code expectation is satisfied (removed).

## Do NOT commit
Leave changes uncommitted. Report: filled scorecard with REAL outputs, files changed, any STOP,
the show-source output sample (proves restringify works), a render-doc output sample (proves the
formatting), any delta.
