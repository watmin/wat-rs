# Arc 143 Slice 6 — Sonnet Brief — define-alias defmacro + apply

**Drafted 2026-05-02 (evening).** Slice 6 of 7. Slice 1 shipped
the three substrate query primitives (lookup-define,
signature-of, body-of) — the introspection bridge from wat
to runtime. This slice ships the userland define-alias macro
+ helpers + applies it for `:wat::core::reduce ↔ :wat::core::foldl`.

**The architectural framing:** wat is closed under macro + AST
construction. Slice 1 added the OBSERVATION primitives. This
slice consumes them: a defmacro that LOOKS UP an existing
function's signature, RENAMES the head, and emits a fresh
define that delegates. Pure userland wat. The alias is the
first user of the reflection foundation; future
reflection-driven macros (sweep generators, spec validators,
doc extractors) reuse the same pattern.

**Goal:** four atomic pieces, composed linearly, shipped in
ONE sweep:

1. `:wat::core::rename-callable-name` (wat helper)
2. `:wat::core::extract-arg-names` (wat helper)
3. `:wat::core::define-alias` (defmacro)
4. `(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)` (the application)

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** —
   the arc's source of truth (now expanded to full
   reflection layer). Read the "Findings" section + the
   "Slice 6" plan + the "Resolution-order semantics" rule.
2. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-1.md`**
   — slice 1's score, with TWO concerns flagged for slice 5/6:
   - Concern 1 (FQDN rendering): synthesised AST renders
     `:Vec<T>` not `:wat::core::Vec<T>`. **This slice tests
     whether that breaks define-alias's generated code.**
     If it does, STOP — surface the failure; don't fix it
     here. Slice 5 owns the FQDN fix.
   - Concern 2 (span discipline): slice 1 used `Span::unknown`
     per brief permission. NOT your concern — slice 5 fixes.
3. **`tests/wat_arc143_lookup.rs`** — the 11 tests slice 1
   shipped. Read them to see the EXACT AST shape the three
   primitives return. Your helpers + macro work against this
   shape.
4. **`src/runtime.rs:5974-6261`** — slice 1's substrate
   implementations of the three primitives (lookup-define,
   signature-of, body-of). Optional read; understand the
   AST construction pattern if you need to debug what
   signature-of returns.
5. **`wat/test.wat:387-403`** — the `:wat::test::make-deftest`
   defmacro. Worked precedent for "macro that builds a
   define using quasiquote + parameter splicing."
6. **`docs/arc/2026/04/091-batch-as-protocol/INSCRIPTION.md`**
   — arc 091 slice 8 documented the quasiquote + struct→form
   semantics. Skim if quasiquote behavior is unclear.
7. **`wat/std/option.wat`** + **`wat/std/result.wat`** —
   stdlib helper precedent. The two new wat helpers + the
   defmacro live in `wat/std/ast.wat` (NEW file) per the
   DESIGN's slice 6 placement.

## What to ship

### Piece 1 — `:wat::core::rename-callable-name` (wat helper)

Signature:
```scheme
(:wat::core::define
  (:wat::core::rename-callable-name
    (head :wat::holon::HolonAST)
    (from :wat::core::keyword)
    (to :wat::core::keyword)
    -> :wat::holon::HolonAST))
```

Takes a signature head AST like
`(:wat::core::foldl<T,Acc> (_a0 :Vec<T>) ... -> :Acc)` and
returns the same head with the function name part of the
first symbol replaced.

Input: `from = :wat::core::foldl`, `to = :wat::core::reduce`.
Output head: `(:wat::core::reduce<T,Acc> (_a0 :Vec<T>) ... -> :Acc)`.

**The first symbol** in the head is `:wat::core::foldl<T,Acc>`
— a SINGLE keyword whose string contains both the name and
type-params. To rename, you need to:
1. Extract the type-params suffix (everything from `<` onward,
   if present)
2. Replace the name part (before `<`) with `to`'s string
3. Re-attach the type-params suffix

If wat has string primitives (`:wat::core::string::contains?`,
`:wat::core::string::split`, `:wat::core::string::concat`),
use them. If they don't exist, **STOP and report** —
"slice 6 needs string primitives; substrate gap surfaced."

### Piece 2 — `:wat::core::extract-arg-names` (wat helper)

Signature:
```scheme
(:wat::core::define
  (:wat::core::extract-arg-names
    (head :wat::holon::HolonAST)
    -> :wat::core::Vector<wat::core::keyword>))
```

Takes a signature head AST and returns a Vec of arg-name
keywords. From `(:foldl<T,Acc> (_a0 :Vec<T>) (_a1 :Acc)
(_a2 :fn(Acc,T)->Acc) -> :Acc)` returns `[:_a0, :_a1, :_a2]`.

Implementation: walk the head's children (it's a
`HolonAST::Bundle`), skip the first child (the function name
symbol), filter for `(arg-name :type)` Bundle pairs (skipping
the `->` symbol and the trailing return-type symbol),
extract the first element of each pair.

### Piece 3 — `:wat::core::define-alias` (defmacro)

Signature:
```scheme
(:wat::core::defmacro
  (:wat::core::define-alias
    (alias-name :AST<wat::core::keyword>)
    (target-name :AST<wat::core::keyword>)
    -> :AST<wat::core::unit>)
  ...)
```

Body composes pieces 1 + 2 + slice 1's signature-of:

```scheme
;; ROUGH SHAPE — adapt to actual wat semantics:
(:wat::core::let*
  (((sig-opt :wat::core::Option<wat::holon::HolonAST>)
    (:wat::core::signature-of target-name))
   ((sig :wat::holon::HolonAST)
    (:wat::core::Option/expect -> :wat::holon::HolonAST
      sig-opt
      "define-alias: target name not found in environment"))
   ((renamed :wat::holon::HolonAST)
    (:wat::core::rename-callable-name sig target-name alias-name))
   ((arg-names :wat::core::Vector<wat::core::keyword>)
    (:wat::core::extract-arg-names sig)))
  `(:wat::core::define
     ,renamed
     (,target-name ,@arg-names)))
```

The expansion takes `(:define-alias :reduce :foldl)` and
produces `(:define (:reduce<T,Acc> (_a0 :Vec<T>) (_a1 :Acc)
(_a2 :fn(Acc,T)->Acc) -> :Acc) (:foldl _a0 _a1 _a2))`.

**HOW the macro receives args**: per typed-macros (058-032),
defmacro params are `:AST<T>` typed. `alias-name` and
`target-name` arrive as AST nodes representing the
keywords the user wrote. Inside the macro body, calling
`(:wat::core::signature-of target-name)` evaluates the AST
to extract the underlying keyword and dispatches.

### Piece 4 — apply

ONE LINE in `wat/std/ast.wat` (or the file where the macro
is defined):

```scheme
(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)
```

After the macro expansion, `:wat::core::reduce` becomes a
real callable in the substrate. Calls to `(:wat::core::reduce
xs init f)` dispatch to the macro-emitted define which
delegates to `(:wat::core::foldl xs init f)`.

## Workflow per piece (THE LOAD-BEARING DISCIPLINE)

For each of the 4 pieces:

1. Add the code (helper / macro / application).
2. Run `cargo test --release --workspace`.
3. Verify: workspace stays green AT LEAST as much as before
   (the 1 pre-existing arc 130 stepping-stone failure may
   FLIP TO PASSING after piece 4 ships — that's the
   confirmation `:reduce` resolves correctly).
4. ONLY THEN move to the next piece.

For pieces 1 and 2 (helpers), add 2-3 unit tests as you go.
For piece 3 (macro), add 1-2 expansion tests. For piece 4
(application), the verification IS the formerly-failing arc
130 test now passing.

**STOP at first red:** if a piece fails, surface the failure
+ stop. Do NOT modify the substrate. Do NOT modify slice 1's
primitives. The failure is data — it surfaces the next gap.

## Constraints

- **Files modified:**
  - `wat/std/ast.wat` (NEW file, all 4 pieces live here)
  - Possibly a new test file (`wat-tests/std/ast.wat` or similar)
  - The expectation: 1 new wat source file + 1 new wat-test
    file. No Rust changes. No substrate changes.
- **No commits, no pushes.**
- **Workspace stays GREEN at piece 3 ship time** (everything
  except the arc 130 stepping stone, which should turn green
  at piece 4 ship). Run `cargo test --release --workspace`
  after each piece.
- **STOP at first red.** Don't grind. Surface + report.

## What success looks like

**Mode A — all 4 pieces ship clean:**
- 2 wat helpers (rename-callable-name + extract-arg-names) defined
- 1 defmacro (define-alias) defined
- 1 apply line ships
- Workspace test green: `cargo test --release --workspace` exit=0
- The arc 130 stepping stone test (`deftest_wat_lru_test_lru_raw_send_no_recv`)
  formerly failing with "unknown function: :wat::core::reduce"
  now PASSES (or fails differently — see Mode B branches).
- Slice 6's own tests (helpers + macro expansion) all pass.

**Mode B variants (each is a clean diagnostic, not a failure):**
- **B-string-primitives**: piece 1 fails because wat lacks
  string primitives needed for the rename. Surface; opens
  a follow-on arc.
- **B-FQDN-rendering**: pieces 1-3 ship; piece 4 emits the
  alias define; type checker rejects the synthesised head
  because `:Vec<T>` isn't a known FQDN type. Surface; slice 5
  takes over.
- **B-other-substrate-gap**: pieces 1-3 ship; piece 4 reveals
  some other subtle issue (e.g., the macro expansion fires
  before signature-of can see the registered foldl primitive,
  or some quasiquote-arity bug). Surface; opens diagnostic
  arc.

**EITHER mode is a successful run.** The reland brief — if
needed — encodes whatever surfaced.

## Reporting back

Target ~250 words:

1. **Piece-by-piece pass/fail roll-up:**
   ```
   Piece 1 (rename-callable-name): PASS — N tests
   Piece 2 (extract-arg-names):    PASS — N tests
   Piece 3 (define-alias macro):   PASS — N expansion tests
   Piece 4 (apply :reduce :foldl): PASS — arc 130 stepping
                                   stone now reports `... ok`
   ```

2. **Final cargo test totals** for `cargo test --release
   --workspace`: passed / failed / ignored.

3. **The macro expansion verbatim** — quote the AST that
   `(:define-alias :reduce :foldl)` expands to. Should look
   like the rough shape in Piece 3's spec.

4. **The four questions verdict** on the wat code YOU wrote:
   - Obvious? Does each helper's failure trace name what
     broke?
   - Simple? Are bodies one outer let* of 3-7 bindings?
   - Honest? Do helper names match bodies?
   - Good UX? Top-down readable; no forward refs?

5. **Honest deltas** — anything you needed to invent or
   diverge from the brief (e.g., wat string primitives didn't
   exist where expected; quasiquote arity required wrapping
   in a different shape; etc.).

6. **File LOC** — wat/std/ast.wat + the test file.

## Sequencing — what to do, in order

1. Read DESIGN.md + SCORE-SLICE-1.md.
2. Read tests/wat_arc143_lookup.rs to see the exact AST
   shape signature-of returns.
3. Read wat/test.wat:387-403 (make-deftest precedent).
4. Read wat/std/option.wat for stdlib placement style.
5. Investigate: do `:wat::core::string::*` primitives exist?
   Quick grep: `grep -n '"wat::core::string' src/runtime.rs`.
   If yes, list what's available. If no, surface as a Mode B
   blocker.
6. Create `wat/std/ast.wat`. Add Piece 1
   (`rename-callable-name`). Add 2-3 tests.
7. Run `cargo test --release --workspace`. Verify Piece 1
   tests pass.
8. Add Piece 2 (`extract-arg-names`). Add 2-3 tests.
9. Run cargo test. Verify Piece 2 tests pass.
10. Add Piece 3 (`define-alias` defmacro). Add 1-2 expansion
    tests.
11. Run cargo test. Verify Piece 3 tests pass + macro
    expansion produces the expected AST.
12. Add Piece 4 (one line: `(:define-alias :reduce :foldl)`).
13. Run cargo test. Verify the arc 130 stepping stone
    (`deftest_wat_lru_test_lru_raw_send_no_recv`) NOW PASSES
    (or fails with a DIFFERENT error — Mode B).
14. Report per "Reporting back."

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.

## Why this slice matters for the chain

This slice TESTS whether the substrate-as-teacher cascade
holds across the full arc 143 stack:
- Slice 1 shipped the introspection primitives
- Slice 6 builds the userland macro atop them
- The arc 130 stepping stone is the END-TO-END verification:
  if `:reduce` resolves correctly, the chain held.

If Mode A: the discipline propagates from substrate to
userland in a single sweep. Arc 130 unblocks. Arc 109 v1
gets one slice closer. Slices 2/3/4/5 ship as parallel
breadth/improvement work; slice 7 closes when all done.

If Mode B: a clean diagnostic surfaces; we open the
appropriate follow-on arc; slice 6 relands; arc 130 still
unblocks once the diagnostic is closed.

This is the full failure-engineering chain in motion. Sonnet
shipped clean for slice 1 from a tight brief; the question
is whether the same discipline holds when the brief
composes 4 sequential userland pieces against the substrate
slice 1 added.
