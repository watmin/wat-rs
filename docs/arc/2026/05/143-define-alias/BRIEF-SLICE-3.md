# Arc 143 Slice 3 — Sonnet Brief — HolonAST manipulation primitives

**Drafted 2026-05-02 (late evening).** Substrate-informed: orchestrator
crawled `holon-rs/src/kernel/holon_ast.rs:51` (HolonAST enum) +
`wat-rs/src/runtime.rs` (existing `:wat::holon::*` primitive
dispatch + Value/HolonAST conversion sites) BEFORE writing this
brief.

**The architectural framing:** HolonAST is a `pub enum` in the
`holon` crate (`holon-rs`). Its variants — `Symbol`, `String`, `I64`,
`F64`, `Bool`, `Atom`, `Bind`, `Bundle`, `Permute`, `Thermometer`,
`Blend` — are accessible via pattern matching from wat-rs. The
substrate already uses this pattern in many places (e.g., `runtime.rs:7906+`).

This slice ships TWO new substrate primitives that operate on HolonAST
heads (signature ASTs). Both are mechanical pattern-match work over
the `Bundle(Vec<HolonAST>)` variant + string surgery on the first
`Symbol` child.

**Goal:** ship two `:wat::runtime::*` primitives:
- `:wat::runtime::rename-callable-name` — substitute the function
  name in a signature head
- `:wat::runtime::extract-arg-names` — return the arg-name keywords
  from a signature head

These unblock slice 6's `define-alias` defmacro.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** —
   read "Findings", the slice-3 plan, and "Resolution-order semantics".
2. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-1.md`** — slice 1
   shipped the synthesized AST shape:
   ```
   Bundle [
     Symbol ":wat::core::foldl<T,Acc>"
     Bundle [Symbol ":_a0", Symbol ":Vec<T>"]
     Bundle [Symbol ":_a1", Symbol ":Acc"]
     Bundle [Symbol ":_a2", Symbol ":fn(Acc,T)->Acc"]
     Symbol "->"
     Symbol ":Acc"
   ]
   ```
   Your primitives operate against this exact shape.
3. **`/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs:51`** —
   the `pub enum HolonAST` definition. Variants include
   `Symbol(Arc<str>)`, `Bundle(Vec<HolonAST>)`, etc. Pattern-match
   directly via the enum.
4. **`src/runtime.rs:5878-5970`** — existing `value_to_watast` +
   `eval_struct_to_form` for the substrate-primitive shape.
5. **`src/runtime.rs:6111-6261`** — slice 1's three primitives
   (`eval_lookup_define`, `eval_signature_of`, `eval_body_of`) +
   their helpers (`function_to_signature_ast`,
   `type_scheme_to_signature_ast`). Place your new primitives near
   these; mirror their argument-validation pattern.
6. **`src/runtime.rs:2411-2413`** — slice 1's dispatch arms. Add yours
   nearby.
7. **`src/check.rs:10997-11021`** — slice 1's scheme registrations.
   Add yours nearby.
8. **`src/check.rs:3126-3163`** — slice 1's type-checker special-case
   for the introspection primitives. Your two primitives need the
   same treatment (HolonAST input + result type).

## What to ship

### Primitive 1 — `:wat::runtime::rename-callable-name`

Signature:
```
(:wat::runtime::rename-callable-name
  (head :wat::holon::HolonAST)
  (from :wat::core::keyword)
  (to :wat::core::keyword))
  -> :wat::holon::HolonAST
```

Takes a signature head AST (`Bundle [Symbol "name<T>", ...]`) and
returns a new head with the function name part of the first Symbol
replaced.

**The first Symbol contains both the name and type-params** — e.g.,
`":wat::core::foldl<T,Acc>"`. To rename:
1. Match on the input HolonAST: must be `HolonAST::Bundle(children)`
   where `children[0]` is `HolonAST::Symbol(s)`. Otherwise return
   error.
2. Parse `s` as `<base-name><optional-type-params-suffix>`:
   - If `s` contains `<`: split at first `<`. Base = before; suffix = `<` onward.
   - Else: base = whole string; suffix = "".
3. Verify base equals the `from` keyword's string (the input keyword
   already has its leading `:`; same for `to`). If not equal, error
   "rename-callable-name: head name does not match `from`".
4. Construct new first symbol: `to + suffix` (e.g.,
   `":wat::core::reduce" + "<T,Acc>"` = `":wat::core::reduce<T,Acc>"`).
5. Construct a new Bundle with `[new_first_symbol, children[1..]]`.

The new Bundle preserves all non-first children unchanged.

### Primitive 2 — `:wat::runtime::extract-arg-names`

Signature:
```
(:wat::runtime::extract-arg-names
  (head :wat::holon::HolonAST))
  -> :wat::core::Vector<wat::core::keyword>
```

Takes a signature head AST and returns a `Vec<keyword>` of the arg
names.

**Algorithm:**
1. Match on input: must be `HolonAST::Bundle(children)`.
2. Skip `children[0]` (the function name symbol).
3. For each remaining child:
   - If it's a `HolonAST::Symbol(s)` and `s == "->"`: STOP (we hit
     the arrow; everything after is the return type).
   - If it's a `HolonAST::Bundle(pair)` where `pair.len() == 2` and
     `pair[0]` is a `HolonAST::Symbol(arg_name)`: collect arg_name
     into the result Vec.
   - Otherwise: skip (not a recognized arg-pair shape).
4. Return the Vec wrapped as `Value::Vec(Arc::new(vec_of_keyword_values))`.

**On the AST shape from slice 1:**

```
Bundle [
  Symbol "..."           ← skip (function name)
  Bundle [Symbol "_a0", Symbol "Vec<T>"]   ← collect ":_a0"
  Bundle [Symbol "_a1", Symbol "Acc"]      ← collect ":_a1"
  Bundle [Symbol "_a2", Symbol "fn(...)"]  ← collect ":_a2"
  Symbol "->"            ← STOP
  Symbol "Acc"           ← (never reached)
]
```

Result: `Vec<keyword>` with three entries: `:_a0`, `:_a1`, `:_a2`.

### Helper functions (Rust-internal)

You'll likely need:
- A pattern-match helper that destructures `Bundle` into the children
  Vec, returning a `RuntimeError` if the input isn't a Bundle.
- A string-split helper for the type-param suffix (split-at-first
  `<`).

Place these near the new eval functions; mirror slice 1's helper
placement (`runtime.rs:5974-6121`).

### Registration

Each primitive registers in two places:

1. **Runtime dispatch** (`src/runtime.rs` near line 2411-2413, in the
   `:wat::runtime::*` block):
   ```rust
   ":wat::runtime::rename-callable-name" => eval_rename_callable_name(args, env, sym),
   ":wat::runtime::extract-arg-names" => eval_extract_arg_names(args, env, sym),
   ```

2. **Type scheme** (`src/check.rs` — register alongside slice 1's
   schemes at lines 10997-11021):
   ```rust
   env.register(":wat::runtime::rename-callable-name".into(), TypeScheme {
       type_params: vec![],
       params: vec![type_keyword("wat::holon::HolonAST"),
                    type_keyword("wat::core::keyword"),
                    type_keyword("wat::core::keyword")],
       ret: type_keyword("wat::holon::HolonAST"),
   });
   // similar for extract-arg-names: takes HolonAST, returns Vec<keyword>
   ```

3. **Type-checker special-case** (`src/check.rs:3126-3163`) — extend
   the existing slice 1 special-case to ALSO handle these two new
   primitives. They share the property that their arg type validation
   needs the same bypass slice 1's introspection primitives use.

## Tests

Add 6-9 tests in a new test file `tests/wat_arc143_manipulation.rs`
(NOT extending the slice 1 file; keep them separate for clean
attribution):

For `rename-callable-name`:
1. Rename a substrate-primitive head (e.g.,
   `(:wat::runtime::rename-callable-name
     (:wat::runtime::signature-of :wat::core::foldl)
     :wat::core::foldl
     :wat::list::reduce)` → returns a HolonAST whose first symbol is
   `:wat::list::reduce<T,Acc>`).
2. Rename a head with NO type-params (e.g., a bare-named function) —
   verify the new symbol has no `<...>` suffix.
3. Error case: input is not a Bundle (e.g., a leaf Symbol).
4. Error case: from-name doesn't match head's base name.

For `extract-arg-names`:
5. Extract from `signature-of :wat::core::foldl` → returns
   `[:_a0, :_a1, :_a2]`.
6. Extract from a head with zero args (e.g., a thunk) → returns `[]`.
7. Extract from a head with `->` and return type after the args —
   verify only arg names are collected, return-type Symbol after `->`
   is NOT included.
8. Error case: input is not a Bundle.

Place tests at the same Rust integration test layer slice 1 used.

## Constraints

- **TWO Rust files modified:** `src/runtime.rs` (eval funcs +
  dispatch arms + helpers) + `src/check.rs` (scheme registrations +
  type-checker special-case extension). ONE NEW test file.
- **No wat files.** No new substrate types. No HolonAST shape
  changes (operate on existing variants).
- **Workspace stays GREEN:** `cargo test --release --workspace`
  exit=0; same baseline + your new tests; same 1 pre-existing LRU
  failure; ZERO new regressions.
- **No commits, no pushes.**

## What success looks like

1. `cargo test --release --workspace`: exit=0; new test file's 6-9
   tests all pass; 1 pre-existing failure unchanged.
2. The two eval functions present in runtime.rs.
3. Dispatch arms + scheme registrations present.
4. Type-checker special-case extended.
5. Verbatim AST shape from a test:
   ```
   (:wat::runtime::rename-callable-name
     (:wat::runtime::signature-of :wat::core::foldl)
     :wat::core::foldl
     :wat::list::reduce)
   ```
   returns `Bundle [Symbol ":wat::list::reduce<T,Acc>", Bundle [...], ...]`
   (verify first symbol's exact string).

## Reporting back

Target ~250 words:
1. **The two eval functions** — name + line range in runtime.rs.
2. **Helper functions** — names + line ranges.
3. **Dispatch arms + scheme registrations** — line numbers.
4. **Verbatim AST output** for a rename test (quote the actual
   shape returned by `rename-callable-name` against
   `signature-of :wat::core::foldl`).
5. **Test totals** — `cargo test --release --workspace` passed /
   failed / ignored. Confirm 1 pre-existing failure unchanged + 0
   new regressions.
6. **Honest deltas** — anything you needed to invent or adapt
   (e.g., a HolonAST helper that doesn't exist; type-keyword
   formatting differences from the brief).
7. **LOC delta** — runtime.rs + check.rs additions + new test file.

## Sequencing — what to do, in order

1. Read DESIGN.md + SCORE-SLICE-1.md + holon_ast.rs:51.
2. Read slice 1's eval functions and helpers
   (`runtime.rs:5974-6261`) for the pattern.
3. Read `src/check.rs:3126-3163` for the type-checker special-case
   shape.
4. Implement the helper functions (Bundle destructuring, string
   split-at-`<`).
5. Implement `eval_rename_callable_name`.
6. Implement `eval_extract_arg_names`.
7. Add dispatch arms in runtime.rs.
8. Add scheme registrations in check.rs.
9. Extend the type-checker special-case to include the two new
   primitives.
10. Add the new test file with 6-9 tests.
11. Run `cargo test --release --workspace`.
12. Report per "Reporting back."

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.
