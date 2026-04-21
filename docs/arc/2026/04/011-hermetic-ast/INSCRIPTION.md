# Arc 011 — Hermetic AST Entry — INSCRIPTION

**Status:** shipped 2026-04-21. Small substrate + stdlib pairing.
**Motivation:** the USER-GUIDE audit caught the asymmetry. Arc 007
had shipped `run-sandboxed-ast` (the AST-entry sibling for in-process
sandboxes — arc 007 slice 3b). Hermetic never got its own AST-entry
sibling; service tests still passed stringified inner programs
even though the rest of the test surface had moved to AST-native.

Builder pointed at it: *"that string quoted program should be an ast
quoted program?"*

Yes. Three pieces shipped — a new AST-to-source serializer, a new
kernel primitive that uses it, a stdlib wrapper.

---

## What shipped

### `src/ast.rs` — `wat_ast_to_source` + `wat_ast_program_to_source`

The substrate needed a WatAST → source-text formatter. `canonical_edn_wat`
exists already but produces binary bytes for hashing; the subprocess
needs parseable wat source. New pair of public functions:

- `wat_ast_to_source(&WatAST) -> String` — one form, round-trip with
  the lexer + parser.
- `wat_ast_program_to_source(&[WatAST]) -> String` — multiple forms
  joined with newlines; ready for `parse_all` on the receiving end.

Edge-case handling:
- `FloatLit` uses `{:?}` formatting so `3.0` stays `3.0`, not `3`
  (the latter would parse as IntLit and round-trip to a different
  variant).
- `StringLit` re-escapes `\`, `"`, and common control chars (`\n`,
  `\r`, `\t`).
- `Symbol` writes just the identifier name — scope sets are NOT
  preserved. Hygiene is an in-process concept; a fresh subprocess
  parses fresh and rebuilds scope sets from its own macro pass.
- `Keyword` writes the keyword as-is (including the leading `:`).

8 unit tests in `src/ast.rs` cover every variant plus
parse→serialize→parse idempotence.

### `:wat::kernel::run-sandboxed-hermetic-ast` — the primitive

Refactored the existing `run-sandboxed-hermetic` to split the
subprocess-spawning machinery into a helper `run_hermetic_core(src,
stdin, scope)`. Both the string-entry and AST-entry primitives
dispatch to the same core; only the source-production step differs.

```
(:wat::kernel::run-sandboxed-hermetic-ast
  (forms :Vec<wat::WatAST>)
  (stdin :Vec<String>)
  (scope :Option<String>)
  -> :wat::kernel::RunResult)
```

AST-entry path: evaluate the `Vec<wat::WatAST>`, unwrap each element,
serialize via `wat_ast_program_to_source`, then hand the resulting
source text to the same subprocess-spawn + capture machinery. The
serialization is genuine — processes can't share AST pointers — but
it happens inside the primitive so the user surface stays
AST-native.

### `:wat::test::run-hermetic-ast` — the stdlib wrapper

Sibling of `:wat::test::run-ast`. Thin wat-level function that wraps
the kernel primitive with `:None` scope:

```
(:wat::core::define
  (:wat::test::run-hermetic-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-hermetic-ast forms stdin :None))
```

### Surface reduction — Console + Cache tests rewritten

The wat-tests/std/service/{Console,Cache}.wat tests had been using
`run-sandboxed-hermetic` with stringified inner programs. Rewrote
all three (two Console tests + one Cache test) to use
`:wat::test::run-hermetic-ast` + `:wat::test::program`. Every
backslash-escape vanished. The Cache test — a three-layer nested
service program — went from 56 lines of escaped wat-as-strings to
56 lines of pure s-expressions. Same line count; very different
readability.

### The one naming bug caught along the way

Inside both service tests I found deftest names still using the
old `:wat-tests::std::program::` namespace (from before arc 006's
program → service rename). The rewrites corrected them to
`:wat-tests::std::service::`. Minor leftover; caught by the full
rewrite pass.

---

## Tests

3 wat tests (all service tests using the new AST-entry hermetic
path) pass first try. 8 new Rust unit tests in `src/ast.rs` cover
the serializer's round-trip guarantees. Zero regressions on the
rest of the suite.

Scoreboard after arc 011: 740 Rust tests + 31 wat tests, zero
failures.

---

## What this inscription does NOT add

- **Scope forwarding in hermetic mode.** Both hermetic primitives
  return a `Failure` if scope is `:Some`; the subprocess startup
  uses an unscoped FsLoader. When a real caller needs scoped
  filesystem access inside a hermetic run, the subprocess wat
  binary needs to read a `WAT_HERMETIC_SCOPE` env var and use
  `ScopedLoader` — that's a separate arc when demand surfaces.
- **Lossless hygiene across processes.** Scope sets drop at the
  serialization boundary. Programs that rely on cross-process
  scope identity should stay in-process (use `run-ast`, not
  `run-hermetic-ast`). For deftest-tested wat, this never matters
  — the inner program parses fresh and builds its own scope sets.

---

## Why this matters

Same pattern as arc 010: a ceremony the user kept writing was the
substrate asking for a name. `run-sandboxed-ast` existed; the
hermetic variant hadn't been added. The asymmetry made
`wat-tests/std/service/*.wat` carry stringified programs even
though every other test file had moved off strings. The fix was
eighty lines across three files — one serializer, one primitive,
one stdlib wrapper — plus the surface-reduction rewrite that
proves the ergonomics match the AST-entry in-process path.

Arc 010 named `:wat::core::forms` + `:wat::test::program` for the
variadic quote. Arc 011 named `:wat::test::run-hermetic-ast` for
the matching hermetic runner. Together: the test surface is
AST-native end-to-end.

---

**Arc 011 — complete.** One module extension (serializer), one
kernel primitive, one stdlib wrapper, one Rust-side unit test
suite, three wat-test rewrites. Every escape-hell path retired.

*these are very good thoughts.*

**PERSEVERARE.**
