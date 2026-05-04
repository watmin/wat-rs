# Arc 144 Slice 4 — BRIEF

**Drafted 2026-05-03 post-arc-146-closure.**

The verification slice. Slice 1 shipped Binding enum + lookup-form
foundation (5 variants). Slice 2 shipped SpecialForm registry.
Slice 3 shipped TypeScheme registrations for hardcoded primitives
+ Mode B-canary diagnostic on the length canary. Arc 146 then
shipped the Dispatch entity + 5 polymorphic Dispatch entities + 5
alias migrations, turning the length canary GREEN. Arc 148 retired
the polymorphic-handler anti-pattern for arithmetic + comparison.

The substrate's uniform-reflection foundation is now structurally
complete. **Slice 4 verifies it works end-to-end across all 6
Binding kinds.**

## What ships

A new test file `tests/wat_arc144_uniform_reflection.rs` with
integration tests covering the full Binding enum coverage that
post-arc-146 + post-arc-148 substrate enables.

### Test coverage (6 kinds × reflection-trio + length canary regression)

For each Binding variant, verify `:wat::runtime::lookup-define`
returns Some + emits the expected declaration head:

| Binding kind | Test target | Expected lookup-define head |
|---|---|---|
| `UserFunction` | a user-defined `(:wat::core::define ...)` | `:wat::core::define` |
| `Macro` | a user-defined `(:wat::core::defmacro ...)` | `:wat::core::defmacro` |
| `Primitive` | `:wat::core::foldl` (TypeScheme primitive) | (no declaration body — primitive sentinel; verify lookup-define returns Some + signature-of returns Some) |
| `SpecialForm` | `:wat::core::if` (slice 2's registry) | (signature sketch from slice 2) |
| `Type` | a user-defined `(:wat::core::struct ...)` | `:wat::core::struct` |
| `Dispatch` | `:wat::core::length` (arc 146 Dispatch entity) | `:wat::core::define-dispatch` |

Plus one **length canary regression test** — replicate the arc 143
slice 6 length test (now-green via arc 146 slice 2):
`(:wat::runtime::define-alias :user::size :wat::core::length)`
followed by `(:user::size hashmap-instance)` returning the
expected size value.

### Reflection-trio coverage

For at least 2 kinds (UserFunction + Dispatch — the most-changed
between slices), ALSO verify `signature-of` returns Some + the
expected shape, AND `body-of` returns the expected shape (Some
for UserFunction; per-kind whatever's appropriate for Dispatch).

### What this slice does NOT ship

- **No `:wat::runtime::doc-string-of` primitive.** Doc strings
  remain unobservable at the wat layer until arc 141 ships the
  primitive + populates the field. Slice 4's "verify doc_string
  field exists + defaults to None" reduces to a Rust-level
  observation: the Binding enum's struct shape enforces the field
  at compile time (already verified by the build). No additional
  test needed for the field's existence.
- **No new substrate primitives.** This is pure verification.
- **No DESIGN updates.** Slice 5 (closure) handles paperwork.

### What this slice DOES ship

- 1 NEW test file: `tests/wat_arc144_uniform_reflection.rs`
- ~6-10 integration tests covering the Binding kinds
- 1 regression test for the length canary
- Per established pattern, mirror `tests/wat_arc144_lookup_form.rs`'s
  shape (wat program with `(:wat::main ...)` printing reflection
  results to stdout via :write-line; Rust harness asserts on the
  output lines).

## Substrate context (read this before writing)

- Binding enum at `src/runtime.rs:7575-7615` — 6 variants, each
  carries `doc_string: Option<String>` (always None as of 2026-05-03)
- `lookup_form` at `src/runtime.rs:6315` (or thereabouts post-arc-144-slice-1)
- Existing tests:
  - `tests/wat_arc144_lookup_form.rs` (9 tests; slice 1) — mirror
    its harness shape
  - `tests/wat_arc144_special_forms.rs` (9 tests; slice 2)
  - `tests/wat_arc144_hardcoded_primitives.rs` (17 tests; slice 3)
  - `tests/wat_arc146_dispatch_mechanism.rs` (7 tests) — already
    covers Dispatch via lookup-define; slice 4 ADDS arc 144's
    "uniform" framing (lookup-form returns Some across kinds)
- The length canary regression target lives in
  `tests/wat_arc143_define_alias.rs` (3/3 passing per arc 146 slice 2 SCORE)

## Pre-flight crawl checklist (sonnet must do)

- [ ] Read `tests/wat_arc144_lookup_form.rs` for harness pattern
- [ ] Read `tests/wat_arc144_special_forms.rs` for SpecialForm test
      shape + verify it covers the slice 2 SpecialForm path
- [ ] Read `tests/wat_arc146_dispatch_mechanism.rs` for Dispatch
      kind tests
- [ ] If existing tests already cover a kind end-to-end via
      lookup-define, ROLL UP the coverage statement in the slice
      4 test file (don't duplicate; reference the existing test
      and add only the gap-coverage tests)

## STOP signals

- If `lookup-form` returns None for a kind that should return
  Some, STOP — that's a substrate gap, not a test gap. Surface
  it as a clean diagnostic; do NOT ship a workaround test.
- If `signature-of` returns the wrong shape for any kind, STOP —
  same shape: substrate gap, surface as diagnostic.
- If you find the length canary is RED (the test in
  `wat_arc143_define_alias.rs` has rotted between arc 146 slice 2
  and now), STOP — substrate-foundation regression that needs
  investigation before slice 4 can verify.

## Output

- ~50-150 LOC test file (sonnet's choice of granularity within
  the coverage scope)
- Honest report following EXPECTATIONS scorecard

## Test execution

`cargo test --release --test wat_arc144_uniform_reflection` should
pass 100%. All 9 substrate-foundation baseline tests should remain
green:
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3
- `wat_arc143_lookup` (whatever it currently is)
- `wat_arc143_manipulation` 8/8
- `wat_arc148_ord_buildout` 46/46
- `wat_polymorphic_arithmetic` 33/33
- `wat_arc150_variadic_define` 16/16

## Why this is the right slice shape now

Per arc 146 + 148 closures, every Binding kind has a substrate-
honest representation. Slice 4 captures that the foundation works
end-to-end — paved-road verification. Slice 5 (closure) writes
the INSCRIPTION + 058 row claiming "all standard polymorphic
surfaces are first-class reflectable entities" — that claim needs
slice 4's verification suite as its evidence.
