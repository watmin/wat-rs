# Arc 122 — Per-test attributes (`:ignore`, `:should-panic`)

**Status:** locked 2026-05-01.

## Provenance

Surfaced 2026-05-01 immediately post-arc-121 during the audit
of "did we satisfy cargo's test contract completely?" Arc 121
landed per-deftest `#[test] fn` emission — 95% of cargo's
contract. The remaining 5% is the per-test attribute
mechanisms: `#[ignore]` and `#[should_panic]`.

User direction:

> if its a new arc, then its a new arc - go make it, then we
> close this out as dependent on the new one — 100% close out

## Goal

Wat deftests can carry the same per-test attributes Rust tests
carry:

- **`#[ignore = "reason"]`** — the test is registered with
  cargo but skipped by default; runs only with
  `cargo test --ignored` or `cargo test -- --include-ignored`.
- **`#[should_panic(expected = "substring")]`** — the test
  is expected to panic; cargo treats the panic as success and
  asserts the panic message contains the expected substring.

## Wat-side syntax

Annotations appear as **sibling forms preceding the deftest**.
They attach to the next deftest the scanner discovers.

```scheme
;; Skip this test entirely:
(:wat::test::ignore "broken on Windows; tracked in issue #123")
(:wat::test::deftest :my::flaky-windows-only
  ...)

;; Expect this test to panic with a specific message:
(:wat::test::should-panic "divide by zero")
(:wat::test::deftest :my::div-by-zero-panics
  (:wat::core::i64::div 1 0))

;; Combined — ignored AND should-panic:
(:wat::test::ignore "intermittent")
(:wat::test::should-panic "expected substring")
(:wat::test::deftest :my::combined
  ...)
```

The annotations are themselves valid wat forms — registered as
no-op verbs in the substrate (take a string, return unit). The
substrate doesn't enforce their semantics; their runtime
presence is irrelevant. They exist as **proc-macro-recognized
syntactic markers** that the discovery scanner attaches to the
next deftest.

## Why sibling forms (not args inside deftest)

Sibling-form annotations:

- **Easier to scan.** The scanner already walks the file looking
  for top-level forms. Recognizing additional heads
  (`:wat::test::ignore`, `:wat::test::should-panic`) is a tiny
  state-machine extension — track "pending annotations" as the
  scanner advances; flush them onto the next deftest discovered.
- **No deftest-macro changes needed.** The wat-substrate-level
  deftest macro is unchanged — it still takes `(:name body)`.
  All annotation handling is proc-macro-level.
- **Backwards compatible.** Existing deftest forms are
  unaffected; new code can opt in as needed.
- **Reads like Rust attributes.** `#[ignore] #[test] fn ...`
  has the attribute BEFORE the test. Sibling-form annotations
  before deftest mirror this naturally.

## Substrate work

### 1. Register no-op verbs

In wat-rs's stdlib (`wat/test.wat` or wherever
`:wat::test::deftest` lives), add:

```scheme
;; Per-test attributes — proc-macro-recognized markers.
;; Substrate-level: take a string, return unit. The marker's
;; runtime presence is irrelevant; arc-121 + arc-122's proc
;; macro reads them at expansion time and emits the matching
;; Rust #[ignore] / #[should_panic] attribute.
(:wat::core::define
  (:wat::test::ignore (_reason :wat::core::String) -> :wat::core::unit)
  ())

(:wat::core::define
  (:wat::test::should-panic (_expected :wat::core::String) -> :wat::core::unit)
  ())
```

These exist purely so the file type-checks. The proc macro is
the consumer.

### 2. Extend the discovery scanner

`crates/wat-macros/src/discover.rs` — `scan_file` becomes
state-aware:

- Track `pending_ignore: Option<String>` and
  `pending_should_panic: Option<String>` as the scanner
  advances.
- On finding `(:wat::test::ignore <string>)`: parse the string
  argument, store as `pending_ignore`.
- On finding `(:wat::test::should-panic <string>)`: same →
  `pending_should_panic`.
- On finding `(:wat::test::deftest :name ...)`: emit a
  `DeftestSite` carrying the pending annotations; clear
  pending state.
- On finding any OTHER form: clear pending state (an annotation
  applies only to the immediately next deftest).

`DeftestSite` extends:

```rust
pub struct DeftestSite {
    pub file_path: PathBuf,
    pub name: String,
    pub ignore: Option<String>,        // arc 122
    pub should_panic: Option<String>,  // arc 122
}
```

### 3. Extend the proc macro emission

`crates/wat-macros/src/lib.rs::test` — for each `DeftestSite`,
emit appropriate attributes:

```rust
#[test]
#[ignore = "reason from wat side"]                    // if site.ignore
#[should_panic(expected = "substring from wat side")] // if site.should_panic
fn deftest_<sanitized>() {
    ::wat::test_runner::run_single_deftest(...);
}
```

The `#[ignore]` and `#[should_panic]` attributes are stable
Rust; libtest handles them natively. No runner changes needed.

### 4. Tests

In `crates/wat-macros/src/discover.rs::tests`:

- `scan_finds_deftest_with_ignore` — annotation before deftest
  attached correctly.
- `scan_finds_deftest_with_should_panic` — same for should_panic.
- `scan_finds_deftest_with_both` — both annotations.
- `scan_clears_pending_on_other_form` — annotation followed by
  unrelated form, then deftest → annotation does NOT attach.
- `scan_orphan_annotation` — annotation at end of file with no
  following deftest → silently ignored (annotation lost).

## Non-goals

- **No new test categories.** Just `#[ignore]` and
  `#[should_panic]`. `#[bench]`, `#[doc]`, `#[cfg(...)]`-style
  conditionals aren't in scope.
- **No combinator macros.** The two annotations stack
  naturally (both before deftest); no shorthand for "ignored
  AND should-panic" is needed.
- **No annotation arguments beyond strings.** Future versions
  might want optional arguments (e.g., timeout duration); arc
  122 keeps the surface minimal.

## Execution checklist

| # | Step | Status |
|---|---|---|
| 1 | Register `:wat::test::ignore` and `:wat::test::should-panic` as no-op `String -> unit` verbs in the appropriate stdlib `.wat` file | pending |
| 2 | Extend `DeftestSite` with `ignore: Option<String>` + `should_panic: Option<String>` fields | pending |
| 3 | Make `scan_file` state-aware — track pending annotations, attach to next deftest, clear on unrelated forms | pending |
| 4 | Add unit tests for the four annotation scenarios | pending |
| 5 | Update `wat::test!` proc macro to emit `#[ignore = "..."]` / `#[should_panic(expected = "...")]` from each site's metadata | pending |
| 6 | Verify on a real test: write a deftest that should-panics, verify `cargo test` reports it as `... ok` (panic occurred as expected) | pending |
| 7 | INSCRIPTION + 058 changelog row | pending |

## Sequencing

Arc 122 closes the per-test-attribute gap that arc 121 left
open. After arc 122 ships:

- Arc 121 closes as 100% complete (no longer "95%, missing
  per-test attributes").
- Existing wat tests unaffected; no migration needed.
- The hanging HolonLRU step-tests can be marked
  `(:wat::test::ignore "blocked on arc 12X — deadlock check")`
  to keep `cargo test --workspace` green while the
  deadlock-check arc develops.

## Cross-references

- `docs/arc/2026/05/121-deftests-as-cargo-tests/DESIGN.md` — the
  arc this one completes.
- `crates/wat-macros/src/discover.rs` — scanner extended here.
- `crates/wat-macros/src/lib.rs::test` — proc macro extended here.
