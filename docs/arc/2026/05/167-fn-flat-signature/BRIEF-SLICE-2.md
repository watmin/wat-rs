# Arc 167 slice 2 — fn-sig vector consumer + walker + defn macro

## Goal

Wire `:wat::core::fn` to consume the new flat-shape signature
`[name <- :T name <- :T] -> :T` (vector + sibling arrow + ret).
Add `BareLegacyFnSignature` walker that fires on legacy nested-sig
`((x :T) (y :T) -> :T)` with a verbose migration message. Update
`:wat::core::defn` macro shape to forward to fn unchanged.

After this slice ships on the branch, the workspace will fail
many tests (every legacy fn/defn callsite). That failure stream IS
slice 3's input — sweep follows on the same branch in a separate
opus run. Main untouched until both slices green.

## Branch + commit policy

- Active branch: `arc-167-slice-2-fn-sig-consumer` (already
  created from main, tracking origin)
- Multiple WIP commits + pushes welcome on the branch for backup
- Do NOT push to main; orchestrator merges slice 2 + slice 3
  atomic to main as one commit after both ship green
- The slice branch will be in a "broken workspace" state at the
  end of slice 2 (intentional — substrate-as-teacher cascade
  setup); slice 3's BRIEF will reference this state

## Background context (read these first)

- `docs/arc/2026/05/167-fn-flat-signature/DESIGN.md` — full arc scope
- `docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-1.md` — slice
  1 closure; the foundation `WatAST::Vector` + parser + error arms
  + walker recursion already shipped at main `7434e4c`
- `docs/SUBSTRATE-AS-TEACHER.md` — failure-engineering discipline
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — failures-are-the-work
- `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md` — closest
  walker-shape precedent (BareLegacyLetStar pattern)
- `wat/core.wat` — current defn macro definition
- `src/runtime.rs` — `eval_fn`, `parse_fn_signature` (lines around
  3722, 3763)
- `src/check.rs` — `parse_fn_signature_for_check`

## Form-shape reference

### New `:wat::core::fn` shape (5 elements at the form level)

```
(:wat::core::fn  ARGS-VECTOR  ->  :RET-TYPE  BODY)
   element 0       element 1   2     3       4
```

- ARGS-VECTOR: `WatAST::Vector` containing flat triples
  `name <- :T name <- :T ...`
  - `name`: `WatAST::Symbol`
  - `<-`: `WatAST::Symbol("<-")` literally
  - `:T`: `WatAST::Keyword` (the type)
- `->`: `WatAST::Symbol("->")` literally
- `:RET-TYPE`: `WatAST::Keyword`
- `BODY`: any AST

Empty-args case: `[]` empty Vector — zero-arity fn.

### New `:wat::core::defn` shape (5 elements)

```
(:wat::core::defn  :NAME  ARGS-VECTOR  ->  :RET-TYPE  BODY)
```

The defn macro forwards to the fn shape directly via `,@rest`
splicing — see "Defn macro" section below.

### Legacy nested-sig (RETIRED; walker fires)

```
(:wat::core::fn  ((x :T) (y :T) -> :T)  body)
                  └── this LIST in args[0] triggers walker
```

The detection signal: at fn position, `args[0]` is a
`WatAST::List` instead of `WatAST::Vector`. The walker fires
`BareLegacyFnSignature` with the verbose migration message.

Same detection for legacy `defn`: macro takes 3 args (name, sig,
body) where sig is a list — caught at the macro shape level OR at
the post-expansion fn position.

## Substrate edits

### 1. `src/runtime.rs::parse_fn_signature` rewrite

Currently parses `((p1 :T1) ... -> :R)` shape. Rewrite to consume
the new fn-form layout. The function is called from `eval_fn`
(line ~3722); update both:

- `eval_fn` now expects `args.len() == 4` (sig-vector, arrow,
  ret-type, body). The 4 args after the head: ARGS-VECTOR, `->`,
  :RET-TYPE, BODY.
- `parse_fn_signature` walks the Vector body in chunks of 3
  (name, `<-`, type), validating each token. Returns
  `(params, param_types, ret_type)` as before.

If `args[0]` is a `WatAST::List` (legacy nested-sig), DO NOT parse
it. Return a clear error or let the walker handle it. Coordinate
with check.rs's walker — runtime path should never see legacy.

Error messages for malformed new shape (not migration; new-shape
typos):
- "fn arg-vector triple at position N must be `name <- :T`; got
  `..."`
- "fn signature missing `->` between args-vector and return type"
- "fn signature missing return-type keyword after `->`"

These help users who write new-shape but get details wrong.

### 2. `src/check.rs::parse_fn_signature_for_check` rewrite

Parallel rewrite. Same shape; same chunked vector consumption;
same error messages. Returns the TypeExpr::Fn the check side uses.

### 3. `src/check.rs` — `BareLegacyFnSignature` walker

Mirror `BareLegacyLetStar` (arc 154) walker pattern:

- New variant: `CheckError::BareLegacyFnSignature { span: Span,
  legacy_sig_text: String }` — the `legacy_sig_text` carries the
  source slice of the legacy sig to enable the migration message
  to show the user's actual code.
- `Display` impl: emit the verbose migration message below.
- Walker `walk_for_legacy_fn_signature` recurses every form,
  detects `(:wat::core::fn LIST ...)` shape (List-as-first-arg
  to fn) — fires fatal with the migration hint.
- Wire into `check_program` via `validate_legacy_fn_signature`,
  ordered before the main inference pass per existing walker
  convention.

The migration message (exact text — do not paraphrase):

```
fn signature must be a vector binding form `[name <- :T name <- :T ...] -> :Ret body`.
Got legacy nested-sig list `((x :T) (y :T) -> :T)`.

Migration:
  - Each `(p :T)` pair becomes `p <- :T` inside a `[...]` vector.
  - The `-> :R` arrow + return type remain siblings of the vector
    (NOT inside the vector).
  - The new shape arrows-as-duals: `<-` consumes (input type),
    `->` produces (output type).

Example:
  Before:  (:wat::core::fn ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64) body)
  After:   (:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 body)

  Defn equivalent (defn is a wat macro composing def + fn):
  Before:  (:wat::core::defn :name ((x :T) -> :T) body)
  After:   (:wat::core::defn :name [x <- :T] -> :T body)

  Zero-arg fn:
  Before:  (:wat::core::fn (-> :wat::core::i64) body)
  After:   (:wat::core::fn [] -> :wat::core::i64 body)
```

This text is the migration brief for slice 3's sweep — keep it
mechanical and unambiguous.

### 4. `wat/core.wat` — defn macro shape change

Current macro:
```scheme
(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    (sig  :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ,name (:wat::core::fn ,sig ,body)))
```

New macro using rest-binder to forward args/arrow/ret/body
to fn unchanged:
```scheme
(:wat::core::defmacro
  (:wat::core::defn
    (name :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::core::nil>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def ,name (:wat::core::fn ,@rest)))
```

Verify the rest-binder type annotation `:AST<wat::core::Vector<wat::core::nil>>`
is what defmacro accepts (per arc 150 variadic-define precedent —
the rest-binder takes a Vector type at the AST layer).

If that annotation doesn't compile, audit the existing variadic
defmacros (`wat/runtime.wat`, `wat/test.wat`) for the right shape
and adjust. Report the actual shape in your final notes.

### 5. Update `tests/wat_arc166_defn.rs` (likely)

Arc 166's tests use the legacy fn-sig shape inside defn calls.
After this slice, those tests will fire the walker. Expected.
Slice 3 sweep will fix them. Do NOT touch them in slice 2.

## Tests

Create `tests/wat_arc167_fn_flat_signature.rs` with cases:

1. **`fn_with_flat_shape_compiles_and_runs`** — `(:wat::core::fn
   [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
   (:wat::core::i64::+,2 x y))` applied as `((fn ...) 2 3)` returns 5.

2. **`defn_with_flat_shape_compiles_and_runs`** — `(:wat::core::defn
   :user::add [x <- :wat::core::i64 y <- :wat::core::i64] ->
   :wat::core::i64 (:wat::core::i64::+,2 x y))` plus a main calling
   `(:user::add 2 3)` returns 5.

3. **`recursive_defn_with_flat_shape`** — fact(5)=120 via flat-shape
   defn. Verifies arc 166's recursive name binding survives the
   shape change.

4. **`zero_arg_fn_with_empty_vector`** — `(:wat::core::fn []
   -> :wat::core::i64 42)` evaluates to 42 when called.

5. **`legacy_nested_sig_fn_fires_walker`** — `(:wat::core::fn
   ((x :wat::core::i64) -> :wat::core::i64) x)` triggers
   `BareLegacyFnSignature`. Assert the error string contains the
   literal `"fn signature must be a vector binding form"`.

6. **`legacy_nested_sig_defn_fires_walker_via_macro`** — legacy-shape
   defn expands to legacy fn → walker fires. Same assertion.

7. **`fn_body_type_mismatch_surfaces`** — flat-shape fn with a
   body whose return type doesn't match the declared `-> :T`.
   Surface `ReturnTypeMismatch` (or equivalent) at the body span.

8. **`malformed_args_vector_clear_error`** — `(:wat::core::fn
   [x <- :wat::core::i64 y] -> :wat::core::i64 x)` (missing
   `<- :T` for `y`). Surface a clear error pointing at position 4
   in the vector.

9. **`reflection_on_flat_defn_resolves`** — `(:wat::runtime::lookup-define
   :user::add)` returns Some(...) after a flat-shape defn registers
   the name. Verifies arc 166's reflection path survives.

Use `startup_ok` / `startup_err` helpers per arc 153/154/155/166
precedent.

## Verification (per scorecard in EXPECTATIONS-SLICE-2.md)

- `cargo build --release --workspace` green
- `cargo test --release --test wat_arc167_fn_flat_signature`:
  9/9 pass
- `cargo test --release --workspace --no-fail-fast`: many failures
  expected (legacy fn/defn callsites in wat-tests + tests + wat
  sources). Count and report. This count is slice 3's input.

## Discipline reminders

- DO NOT push to main; only push to the slice branch
- DO NOT touch wat-source files (`wat/*.wat` except `wat/core.wat`
  for defn macro, `wat-tests/*.wat`, `tests/wat_*.rs`) other than
  `wat/core.wat` for the defn macro AND the new test file. The
  sweep is slice 3's territory.
- DO NOT delete walker scaffolding even after legacy sites
  surface; walker stays alive through slice 3; retirement is
  slice 4
- If the rest-binder type annotation `:AST<wat::core::Vector<wat::core::nil>>`
  doesn't match defmacro's actual variadic shape, audit existing
  variadic defmacros and use what works; report your finding
- If a substrate decision arises that isn't covered, STOP and
  report; orchestrator decides

## Report shape

When complete, report:

1. cargo test summary for `wat_arc167_fn_flat_signature` only
   (passed/failed)
2. cargo test summary for the FULL workspace (passed/failed —
   the failure count in the workspace is slice 3's input; expect
   tens to hundreds of failures)
3. Each substrate site you edited (file + line range) with a
   one-line description
4. Final defn macro shape (paste it from `wat/core.wat`)
5. Walker shape (paste the `BareLegacyFnSignature` error message
   from your `Display` impl)
6. Test names + pass/fail status of the 9 cases
7. Honest deltas — substrate decisions you made beyond this
   BRIEF, especially around (a) how `eval_fn` reshapes for the new
   arity, (b) the rest-binder type annotation that worked, (c) any
   walker-coverage gaps detected via failing-but-shouldn't-fire
   tests
8. Branch state confirmation
9. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS. If you exceed the upper bound still iterating,
STOP and report current state.
