# Arc 150 Slice 1 — Sonnet Brief — Implement variadic `:wat::core::define`

**Drafted 2026-05-03.** Foundation slice. Substrate-informed:
orchestrator's pre-flight crawl confirmed (a) `:wat::core::define`
has NO rest-param support (only fixed-arity); (b) `:wat::core::defmacro`
ALREADY supports variadic via `MacroDef.rest_param`; (c) the arc 148
slice 4 architecture requires variadic define to express
`(:wat::core::+ & (xs :Vector<numeric>) -> :numeric) (reduce ...)`.

FM 9 baseline confirmed pre-spawn (post-slice-5):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3
- `wat_polymorphic_arithmetic` 20/20
- `wat_arc148_ord_buildout` 46/46

**Goal:** add `& rest` rest-param support to `:wat::core::define`,
mirroring `:wat::core::defmacro`'s exact syntax. Variadic functions
become first-class wat constructs. NO change to existing strict-arity
defines (rest_param field is `Option<String>`, defaulting to None).

**Working directory:** `/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`docs/arc/2026/05/150-variadic-define/DESIGN.md`** — full design
   context + the architecture this slice implements.
2. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — discipline. § FM 9
   (baselines done); § 12 (foundation work; eliminate failure domains).
3. **`tests/wat_variadic_defmacro.rs`** — the existing variadic
   defmacro test suite. Mirrors the patterns slice 1 will need to
   support for variadic define. Read it for shape + structure.
4. **`src/macros.rs:68-95`** — `MacroDef` struct (template for
   `Function`'s new fields).
5. **`src/macros.rs:380-450`** — `parse_defmacro_signature`'s
   rest-param parsing logic (pattern to mirror for define).
6. **`src/macros.rs:551-620`** — `expand_macro_call`'s arity
   handling for variadic macros (template for `apply_function`'s
   variadic arity logic).
7. **`src/runtime.rs:499-525`** — `Function` struct definition (the
   site of new fields).
8. **`src/runtime.rs:1681-1820`** — `parse_define_form` +
   `parse_define_signature` (the parser to extend).
9. **`src/runtime.rs:12865-12920+`** — `apply_function`'s arity check
   + arg binding (the runtime to extend).
10. **`src/check.rs:63-110`** — `TypeScheme` struct (the type-
    inference scheme to extend).
11. **`src/check.rs:8397-8430`** — `derive_scheme_from_function` (the
    derivation to extend).
12. **`src/check.rs` — call-site inference for Function callees**
    (locate via `grep -n "Function" src/check.rs | head -20`; needs
    extension for variadic call sites).

## What ships

### 1. Function struct extension (`src/runtime.rs:499`)

Add two fields to `pub struct Function`:

```rust
pub rest_param: Option<String>,
pub rest_param_type: Option<crate::types::TypeExpr>,
```

Both fields default to `None` for existing strict-arity defines (no
behavior change to current tests). When a variadic define is parsed,
`rest_param` carries the rest-binder name; `rest_param_type` carries
the declared `Vector<T>` type expression.

### 2. parse_define_signature (`src/runtime.rs:1736`)

Extend to accept the `& (name :Type)` pattern after fixed params,
mirroring `src/macros.rs:380-450`. Validation:

- `&` must be followed by exactly one binder (a list of `(name :Type)`)
- Type is REQUIRED (not optional); rest-param without type is a
  parse error
- Type MUST be a `Vector<T>` shape — the substrate carries rest-args
  as a Vector at runtime
- `&` may appear at most once; multiple `&` markers are a parse error
- After `&` + one binder, the rest of the signature is the `-> :T`
  return type (no additional fixed params)

Populate the new `rest_param` + `rest_param_type` fields on the
parsed Function.

### 3. apply_function (`src/runtime.rs:12865`)

Extend the arity check at the top of `apply_function`'s loop:

```rust
// Pseudocode — adapt to the actual locals
let fixed_arity = cur_func.params.len();
let actual_arity = cur_args.len();

if cur_func.rest_param.is_none() {
    // Existing strict-arity behavior
    if actual_arity != fixed_arity {
        return Err(ArityMismatch { expected: fixed_arity, got: actual_arity, ... });
    }
} else {
    // Variadic: accept actual_arity >= fixed_arity
    if actual_arity < fixed_arity {
        return Err(ArityMismatch { expected: fixed_arity, got: actual_arity, ... });
    }
}
```

After the check, when binding args to the call env:
- Bind the first `fixed_arity` args to `cur_func.params` as today
- Collect the remaining args into `Vec<Value>`, wrap in
  `Value::Vec(Arc::new(rest))`, bind to `cur_func.rest_param.unwrap()`

### 4. TypeScheme + derive_scheme_from_function (`src/check.rs`)

Extend `TypeScheme` (around line 63) with a parallel rest-shape:

```rust
pub rest_param_type: Option<TypeExpr>,
```

Update `derive_scheme_from_function` (around line 8397) to populate
the new field from `Function.rest_param_type`.

### 5. Call-site type checking (`src/check.rs`)

When a call site invokes a Function whose scheme has
`rest_param_type.is_some()`:

- Accept `args.len() >= params.len()` (NOT strict equality)
- Type-check the first `params.len()` args against `params`
- Type-check each rest-arg against the element type T extracted from
  the rest_param_type (which is `Vector<T>`); each rest-arg must
  unify with T
- Return type unchanged (still from the scheme)

Locate the relevant call-site inference logic via grep on
`infer_list` or wherever `params.len()` is checked for Function
callees. The arity check sites need extension; the type-checking
loop needs to handle rest-args.

### 6. Tests (`tests/wat_arc150_variadic_define.rs`)

NEW file. Mirror `tests/wat_variadic_defmacro.rs`'s shape (use
`startup_from_source` + `invoke_user_main`). Coverage:

- `variadic_define_with_zero_rest_args` — variadic define called
  with exactly `params.len()` args; rest binds to empty Vector
- `variadic_define_with_one_rest_arg`
- `variadic_define_with_many_rest_args`
- `variadic_define_no_fixed_params` — only `& (rest :Vec<T>)`
- `arity_error_below_fixed` — caller passes fewer than fixed_arity
  args
- `type_error_rest_arg` — rest-arg whose type doesn't unify with T
- `signature_of_variadic_returns_rest_shape` — arc 144 reflection
  primitive surfaces the variadic info correctly
- `variadic_define_uses_reduce_over_rest` — the canonical pattern
  (define + reduce over rest args; what arc 148 slice 4 needs)

Plus negative tests:
- `parse_error_double_ampersand` — `& x & y` is rejected at parse
- `parse_error_rest_without_type` — `& (rest)` without `:Type` is
  rejected
- `parse_error_rest_followed_by_fixed_param` — `& (rest :T) (x :U)`
  is rejected

## What this slice does NOT do

- NO change to `:wat::core::lambda` (lambdas stay fixed-arity)
- NO change to `:wat::core::defmacro` (already variadic; UNCHANGED)
- NO new substrate primitives registered
- NO retirement of any existing form
- NO touching arc 148's plan (that resumes after arc 150 closes)

## STOP at first red

If you discover the rest_param_type CAN'T be a parametric `Vector<T>`
without major check-side surgery (e.g., the existing TypeScheme
unification machinery doesn't handle rest-args cleanly), STOP and
report. Don't improvise a workaround.

If a baseline test fails post-extension (existing strict-arity define
suddenly fails because the new code path interferes), STOP and report.
The contract is "rest_param.is_none() means existing behavior";
violating that is a substrate bug.

If you find that `apply_function`'s tail-call loop has subtle
interactions with rest-arg binding (e.g., the rest Vec needs to be
re-collected on each tail iteration), surface as honest delta.

## Source-of-truth files

- `src/macros.rs:68, 380-450, 551-620` — defmacro variadic (template)
- `src/runtime.rs:499, 1681-1820, 12865+` — Function + parse_define
  + apply_function (sites to extend)
- `src/check.rs:63, 8397+` — TypeScheme + derive_scheme + call-site
  inference (sites to extend)
- `tests/wat_variadic_defmacro.rs` — the test pattern to mirror

## Honest deltas

If you find:
- Existing tests that break because of the new rest-param handling
  (suggesting a coupling we missed)
- A reflection primitive (`signature-of`, `body-of`, `lookup-define`)
  that needs updating to surface the new variadic info
- A scope-deadlock or scope-leak interaction with the rest Vec binding
- Any place where the rest_param needs to integrate with `:wat::core::let*`
  destructuring or pattern matching

Surface as honest delta.

## Report format

After shipping:

1. Total Function field additions (should be 2)
2. Total parse_define_signature changes
3. Total apply_function changes
4. Total check.rs changes (TypeScheme + derive + call-site)
5. New test file line count + count of test functions
6. All tests passing (baselines + new)
7. Any honest deltas surfaced

Time-box: 120 min wall-clock. Predicted Mode A 50-80 min. Substrate
work spans multiple files; one cohesive slice.

## What this unlocks

**Arc 148 slice 4** can now ship its variadic arithmetic surface as
a wat-level function reducing over the binary Dispatch — the locked
DESIGN's intent becomes implementable.

Beyond arc 148: any future variadic surface (format, log, pipe,
test harness extensions) becomes expressible without falling back
to defmacro-with-runtime-branching or Rust-only primitives.

The substrate becomes consistent — defmacros and defines both
support `& rest`. The arbitrary asymmetry is closed.
