# Arc 143 Slice 6 — Sonnet Brief — `:wat::runtime::define-alias` defmacro

**Drafted 2026-05-02 (late evening)** in parallel with slice 3 sweep.
Substrate-informed: orchestrator crawled the macro precedents
(`wat/test.wat:387-403`, `wat/holon/Subtract.wat`, etc.), the
substrate primitives shipped in slices 1+2+3, and the type-registry
canonicalization (substrate stores bare-name heads; parser accepts
bare names) BEFORE writing this brief.

**The architectural framing:** slices 1+2+3 ship the substrate
foundation. Slice 1 added the runtime introspection point-lookups
(`signature-of`, `lookup-define`, `body-of`). Slice 2 enabled
computed unquote in defmacro bodies. Slice 3 added HolonAST
manipulation primitives (`rename-callable-name`,
`extract-arg-names`). Slice 6 composes them into ONE userland
defmacro: `:wat::runtime::define-alias`.

**Goal:** ship `:wat::runtime::define-alias` as a defmacro in a NEW
top-level wat file `wat/runtime.wat`. The macro takes two keyword
arguments (alias-name + target-name) and emits a fresh
`:wat::core::define` whose head copies the target's signature with
the alias name substituted, and whose body delegates to the target.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** — read the
   slice-6 plan + the post-slice-3 architecture.
2. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-1.md`** — slice 1
   primitives' AST shapes.
3. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-2.md`** — slice 2
   computed-unquote semantics and the head-is-Keyword heuristic.
4. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-3.md`** (read
   when shipped) — slice 3 manipulation primitives' exact APIs.
5. **`tests/wat_arc143_lookup.rs`** — slice 1 tests; verify the
   AST shape `signature-of` returns.
6. **`wat/test.wat:387-403`** — `:wat::test::make-deftest` defmacro,
   the worked precedent for "macro that builds a define using
   quasiquote." Note the `:AST<T>` parameter typing.
7. **`wat/holon/Subtract.wat`** — simple defmacro example for
   placement style + header comments.
8. **`src/stdlib.rs:100-130`** — how top-level wat files
   (`wat/test.wat`, `wat/console.wat`, etc.) are registered. New
   `wat/runtime.wat` registers similarly.

## What to ship

### Piece 1 — Create `wat/runtime.wat` (NEW top-level file)

File header (mirror `wat/test.wat`'s style — short doc comment +
section markers):

```scheme
;; wat/runtime.wat — :wat::runtime::* macros.
;;
;; Runtime-discovery + reflection-driven macros built atop the
;; substrate primitives shipped in arcs 143 slices 1+2+3.
```

### Piece 2 — `:wat::runtime::define-alias` defmacro

Signature (typed-macros per 058-032):

```scheme
(:wat::core::defmacro
  (:wat::runtime::define-alias
    (alias-name :AST<wat::core::keyword>)
    (target-name :AST<wat::core::keyword>)
    -> :AST<wat::core::unit>)
  ...body...)
```

Body composition (using slices 1+2+3's primitives):

```scheme
;; Use computed unquote (slice 2) to call substrate primitives
;; at expand-time, splicing the results into a generated define.
;;
;; signature-of returns Option<HolonAST>; we Option/expect to a HolonAST,
;; then rename + extract-arg-names from it.
`(:wat::core::define
   ,(:wat::runtime::rename-callable-name
      (:wat::core::Option/expect -> :wat::holon::HolonAST
        (:wat::runtime::signature-of target-name)
        "define-alias: target name not found in environment")
      target-name
      alias-name)
   (,target-name ,@(:wat::runtime::extract-arg-names
                     (:wat::core::Option/expect -> :wat::holon::HolonAST
                       (:wat::runtime::signature-of target-name)
                       "define-alias: target name not found in environment"))))
```

The macro expands `(:wat::runtime::define-alias :reduce :foldl)` to:

```scheme
(:wat::core::define
  (:reduce<T,Acc>
    (_a0 :Vec<T>)
    (_a1 :Acc)
    (_a2 :fn(Acc,T)->Acc)
    -> :Acc)
  (:foldl _a0 _a1 _a2))
```

(or whatever the exact shape is per slice 3's `rename-callable-name`
output and slice 1's `signature-of` output.)

### Piece 3 — Register `wat/runtime.wat` in `src/stdlib.rs`

Add an entry for the new file alongside the existing
`wat/test.wat`, `wat/console.wat`, etc. entries (around
`src/stdlib.rs:100-130`).

The exact registration shape mirrors the existing entries — a
struct/tuple with `path: "wat/runtime.wat"` + `source:
include_str!("../wat/runtime.wat")`.

## Tests

Add 2-3 tests in a new test file `tests/wat_arc143_define_alias.rs`
(mirror slice 1 + 3's test placement convention):

1. **Macro expansion test** — verify `(:wat::runtime::define-alias
   :my::test-alias :wat::core::foldl)` in a deftest body expands to
   a `(:wat::core::define ...)` form that itself parses + type-checks.
2. **Functional test** — define a user function `:user::triple` that
   uses `:my::test-alias` (the alias). Verify the call resolves
   correctly (delegates to foldl).
3. **Error case** — `(:wat::runtime::define-alias :a :name-that-does-not-exist)`
   surfaces a clear error at expand-time (the
   `define-alias: target name not found` message from `Option/expect`).

## Constraints

- **Files modified:** `wat/runtime.wat` (NEW), `src/stdlib.rs` (1
  registration entry), `tests/wat_arc143_define_alias.rs` (NEW).
  No substrate Rust changes.
- **`wat/std/` is OFF LIMITS.** Arc 109 is killing the `:wat::std::*`
  namespace. The new file goes at `wat/runtime.wat` (top-level).
- **Workspace stays GREEN:** `cargo test --release --workspace`
  exit non-zero only because of the 1 pre-existing arc 130 LRU
  failure; new tests pass; ZERO new regressions.
- **No commits, no pushes.**

## What success looks like

**Mode A — clean ship:**
- The defmacro is registered in the macro registry at stdlib load.
- The 2-3 new tests pass.
- The macro expansion test verifies the emitted define's exact AST
  shape.
- The functional test calls the alias and gets the expected result.

**Mode B — FQDN gap surfaces:**
- The macro expansion succeeds, BUT the emitted define fails to
  parse / type-check because the head's bare type names
  (`:Vec<T>`, etc.) don't resolve in the parsing context. STOP and
  report exactly what failed.
- This means slice 5a (FQDN rendering fix) is needed BEFORE slice 6
  ships. The brief said this might happen; clean diagnostic.

**Mode B — type-checker special-case gap:**
- The macro's body uses `signature-of`, `rename-callable-name`,
  `extract-arg-names`, `Option/expect`. If the type-checker rejects
  the body's expression at expand-time (e.g., the special-case for
  the runtime primitives doesn't extend to nested calls), STOP and
  report.

EITHER mode is a clean run of the discipline.

## Reporting back

Target ~250 words:

1. **`wat/runtime.wat` content** verbatim (the file is small; quote
   it).
2. **`src/stdlib.rs` change** — line numbers + the new entry.
3. **The macro expansion verbatim** — what does
   `(:wat::runtime::define-alias :my::alias :wat::core::foldl)`
   expand to? Quote the AST.
4. **Test file content** — name + count of tests.
5. **Test totals** — `cargo test --release --workspace` passed /
   failed / ignored. Confirm 1 pre-existing failure unchanged + 0
   new regressions.
6. **Honest deltas** — anything you needed to invent or adapt
   (e.g., the macro body's quasiquote shape, `Option/expect` usage,
   any FQDN gap if Mode B).

## Sequencing — what to do, in order

1. Read DESIGN.md + SCORE-SLICE-1/2/3 + slice 3's test file (when
   shipped).
2. Read `wat/test.wat:387-403` for the make-deftest precedent.
3. Read `wat/holon/Subtract.wat` for placement style.
4. Read `src/stdlib.rs:100-130` for registration shape.
5. Create `wat/runtime.wat` with the header + the defmacro.
6. Update `src/stdlib.rs` with the registration entry.
7. Create `tests/wat_arc143_define_alias.rs` with 2-3 tests.
8. Run `cargo test --release --workspace`.
9. If Mode A: report success; STOP.
10. If Mode B: report exactly what failed + hypothesis; STOP. Don't
    grind.

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.

## Why this slice matters

Slice 6 is the END-TO-END test of arc 143's substrate-as-teacher
cascade. Slices 1+2+3 shipped the foundation; slice 6 is the FIRST
USER. If it ships clean, the whole reflection layer works as
designed.

Mode A ships clean → slice 7 (apply :reduce/:foldl) trivially → arc
130 stepping stone unblocks → arc 109 v1 closes.

Mode B with FQDN gap → slice 5a opens, fixes, slice 6 relands.

Mode B with macro-body-typing gap → slice 5b opens, fixes, slice 6
relands.

Either path is the discipline working. The value is the calibration.
