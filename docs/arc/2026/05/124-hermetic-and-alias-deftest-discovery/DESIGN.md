# Arc 124 — Hermetic + alias deftest discovery

**Status:** **shipped + closed 2026-05-01.** See INSCRIPTION.md
for the close-out summary, what got surfaced, and the four
questions. The DESIGN below is the as-drafted record kept
verbatim for archaeology.

## Provenance

Arc 121 shipped per-deftest `#[test] fn` emission by scanning
`.wat` source for literal `(:wat::test::deftest <name> ...)`
forms. Arcs 122 + 123 added per-test attributes (`:ignore`,
`:should-panic`, `:time-limit`) attached as sibling annotations.

A stepping-stone agent kicked off mid-arc-119 surfaced a gap:
the brief instructed the agent to use `deftest-hermetic` (so
service-spawning tests run in a forked subprocess), but the
arc-121 scanner only matches `(:wat::test::deftest ...)`. The
existing `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
uses six tests — none of them are discovered:

```bash
$ cargo test --release -p wat-holon-lru -- --list | grep HologramCacheService
# empty — six tests live in the file; the scanner sees none
```

The file's preamble:

```scheme
(:wat::test::make-deftest :deftest-hermetic
  ((:wat::core::define ...)))

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
  (...))
```

`(:deftest-hermetic ...)` is a make-deftest-registered alias.
The wat-side macro expansion at type-check time turns each
alias call into `(:wat::test::deftest-hermetic ...)`, but the
proc-macro scanner runs at compile time and only sees the
literal source text. It doesn't know `:deftest-hermetic`
expands to anything test-related.

User direction (2026-05-01):

> make this a new arc and address it - now

## Goal

The proc-macro scanner discovers four shapes:

1. `(:wat::test::deftest <name> <prelude> <body>)` — already shipped
2. `(:wat::test::deftest-hermetic <name> <prelude> <body>)` — NEW
3. `(:alias <name> <body>)` where alias was declared via
   `(:wat::test::make-deftest :alias <prelude>)` — NEW
4. `(:alias <name> <body>)` where alias was declared via
   `(:wat::test::make-deftest-hermetic :alias <prelude>)` — NEW

Each becomes its own `#[test] fn`. Annotations (`:ignore`,
`:should-panic`, `:time-limit`) attach to all four shapes
identically.

## Non-goals

- **No runner change.** The wat-side macro expansion already
  encodes hermetic vs in-process — `deftest-hermetic` expands
  to a `:wat::core::define` calling `run-sandboxed-hermetic-ast`;
  `deftest` calls `run-sandboxed-ast`. Both produce a function
  returning `:wat::test::TestResult`. `run_single_deftest`
  looks up the function in the frozen symbols and calls it;
  it doesn't care which inner runner the body chose.
- **No new wat syntax.** All four shapes already exist; the
  arc just teaches the scanner to recognize the existing
  forms.
- **No cross-file alias tracking.** Each `.wat` file's aliases
  are scanner-local. Aliases defined in file A don't affect
  file B's scanner pass. (Mirrors wat's runtime defmacro
  scope — defmacros are file-scoped after freeze.)

## Architecture

One file changes: `crates/wat-macros/src/discover.rs`.

### Scanner state

Today's scanner has three pieces of pending state
(`pending_ignore`, `pending_should_panic`, `pending_time_limit_ms`).
Arc 124 adds one more — a per-file alias table:

```rust
struct AliasSet {
    /// Maps alias keyword (e.g. ":deftest-hermetic") to the
    /// hermeticity it inherits. Both flavors register here;
    /// the scanner emits the same DeftestSite either way.
    aliases: HashMap<String, ()>,
}
```

The scanner only needs the SET of registered aliases — it
doesn't need to track hermeticity at the scanner layer
because the runner doesn't care. Keeping `HashMap<String, ()>`
(not `HashSet<String>`) leaves room for future per-alias
metadata if it surfaces.

### Top-level form recognition

Today's scanner matches `head_str` against four literal
keywords (`:ignore`, `:should-panic`, `:time-limit`,
`:deftest`). Arc 124 adds three more match arms:

| Head | Action |
|---|---|
| `:wat::test::deftest-hermetic` | same as `:wat::test::deftest` — read next keyword as test name, emit a site with pending annotations |
| `:wat::test::make-deftest` | read next keyword as alias name, register in alias table; clear pending annotations (annotations don't carry to the alias-defining call) |
| `:wat::test::make-deftest-hermetic` | same as `:wat::test::make-deftest` |

Plus a CATCH-ALL: when a top-level form's head keyword is in
the alias table, treat it as a deftest call (read next
keyword as test name, emit a site).

### Order matters

Aliases must be registered BEFORE the deftest calls that use
them. wat-source files always declare `(:wat::test::make-deftest
:alias ...)` at the top of the file before the alias's first
use. The scanner walks top-to-bottom, so this is naturally
respected — see `scan_alias_must_be_declared_before_use` test.

A reverse-order file would silently drop the calls (they'd
fall into the "unknown head, clear pending annotations"
branch). This matches wat's runtime behavior — defmacros
called before declaration error at type-check time. The
scanner's silent drop is conservative; the error surfaces
later at the wat-level type-check.

### What the proc-macro emits

Identical to today's emission. The scanner produces a
`DeftestSite` regardless of whether the discovery shape was
`deftest`, `deftest-hermetic`, or an alias. The emitted
`#[test] fn` calls `run_single_deftest` with the deftest's
keyword name; the runner looks up the registered function;
the wat-side macro expansion handled hermetic dispatch at
freeze time. **Total runtime delta from arc 124: zero.**

## Test plan

Add unit tests to `discover.rs::tests`:

1. `scan_finds_deftest_hermetic` — direct
   `(:wat::test::deftest-hermetic :name ())` discovered.
2. `scan_alias_via_make_deftest` — `(:wat::test::make-deftest
   :alias ())` then `(:alias :test-name ())` discovered.
3. `scan_alias_via_make_deftest_hermetic` — same with the
   hermetic factory.
4. `scan_alias_with_pending_annotations` — annotations attach
   to alias calls correctly.
5. `scan_alias_does_not_register_on_make_deftest` —
   annotations preceding `make-deftest` are dropped (don't
   leak to the alias declaration call itself, since aliases
   aren't tests).
6. `scan_unknown_alias_silently_dropped` — `(:not-an-alias
   :name ())` doesn't get treated as a test.
7. `scan_alias_must_be_declared_before_use` — alias calls
   before their `make-deftest` declaration are silently
   dropped.
8. `scan_multiple_aliases_in_one_file` — two distinct aliases
   coexist.
9. `scan_finds_mixed_shapes` — file with all four shapes;
   all discovered.

End-to-end:

10. `cargo test --release -p wat-holon-lru -- --list` shows
    six `HologramCacheService` tests AFTER arc 124 ships
    (currently zero).

## Workspace hygiene — keep cargo test green

Discovering the six `HologramCacheService` tests will
surface the arc-119 deadlocks at runtime. The honest
sequence:

1. Ship arc 124 scanner extension. Six tests now discoverable.
2. The four arc-119-recovered hanging tests (steps 3-6) stay
   hanging until arc 119 closes; mark each with
   `(:wat::test::time-limit "200ms")` so cargo test fails
   cleanly with a timeout panic instead of hanging the test
   binary.
3. Steps 1 and 2 (`test-step1-spawn-join`,
   `test-step2-counted-recv`) likely pass; they don't carry
   the deadlocked Put-ack scenario.

Step 2 above is workspace-hygiene work, not arc-124-proper.
It lands in the same arc commit because `no-broken-commits`
applies.

## Execution checklist

| # | Step | Status |
|---|---|---|
| 1 | Extend `discover.rs` scanner — add `:wat::test::deftest-hermetic` arm; add `:wat::test::make-deftest` + `:wat::test::make-deftest-hermetic` arms (register alias); add catch-all for known aliases | pending |
| 2 | Add 9 scanner unit tests covering all four shapes, alias-before-use, mixed shapes | pending |
| 3 | `cargo test -p wat` (run scanner unit tests) — confirm green | pending |
| 4 | `cargo test --release -p wat-holon-lru -- --list` — confirm 6 new HologramCacheService tests now appear | pending |
| 5 | Apply `(:wat::test::time-limit "200ms")` to the four arc-119-deadlocked HologramCacheService tests; leave step1 + step2 unannotated (they should pass) | pending |
| 6 | `cargo test --release --workspace` — confirm no hangs (timeouts fire cleanly), step1 + step2 pass, step3-6 fail with timeout panic | pending |
| 7 | INSCRIPTION + 058 changelog row | pending |

## Discipline anchors

- **No new wat syntax.** All four discovery shapes already
  exist as wat-side forms.
- **No new runner code.** The wat-side macro expansion
  handles hermetic dispatch.
- **Cargo-native parity.** `cargo test --list` shows every
  deftest regardless of source-text shape; `cargo test
  <substring>` filters identically. The tests are
  first-class regardless of declaration shape.
- **No broken commits.** Annotation passes for the existing
  hanging tests land in the same commit.

## Cross-references

- `docs/arc/2026/05/121-deftests-as-cargo-tests/DESIGN.md` —
  the per-deftest emission machinery this arc extends.
- `docs/arc/2026/05/122-per-test-attributes/DESIGN.md` —
  annotations that apply uniformly across all four shapes.
- `docs/arc/2026/05/123-time-limit/DESIGN.md` — the
  annotation that makes step 5 above honest.
- `crates/wat-macros/src/discover.rs` — the only file that
  changes.
- `wat/test.wat` lines 304-414 — the four wat-side macros
  this arc teaches the scanner to recognize.
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
  — the file whose tests have been silently invisible since
  arc 121 shipped.
