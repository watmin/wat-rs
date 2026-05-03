# Arc 143 Slice 1 — Sonnet Brief — Three substrate query primitives

**Drafted 2026-05-02 (evening).** Surfaced from arc 130 slice 1
RELAND's diagnostic at Layer 1 (substrate's `:wat::core::reduce`
→ "unknown function"; the empirically-observed bias to reach
for `reduce` over `foldl`).

**The architectural framing:** the wat substrate is closed under
macro + AST construction. What it lacks is the ability to
OBSERVE the runtime's own symbol table — every existing
introspection primitive (`struct->form`, arc 037's
`statement-length`) operates on data the user already holds.
This slice ships the three query primitives that bridge wat
code to the runtime's bindings: `lookup-define`, `signature-of`,
`body-of`. Once these exist, `define-alias` (slice 3), the
helpers (slice 2), and the actual `(:define-alias :reduce
:foldl)` (slice 4) are pure wat.

**Goal:** ship three new substrate primitives in a single sweep.
Mechanical to ship together because they share lookup
machinery + AST construction helpers.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** — the
   arc's source of truth. Read the "Findings" section
   (resolves Q1 + Q2 with concrete pointers to runtime.rs +
   check.rs lines). Read "Resolution-order semantics" for
   the lookup precedence (env → schemes → None).
2. **`src/runtime.rs:499-510`** — the `Function` struct.
   Note the fields preserved at define-time: `name`,
   `params` (NAMES), `type_params`, `param_types`,
   `ret_type`, `body`. All needed AST data is present.
3. **`src/runtime.rs:563`** — `Environment::lookup(name) ->
   Option<Value>`. The user-define lookup path.
4. **`src/runtime.rs:5891+`** — `eval_struct_to_form` (the
   `:wat::core::struct->form` primitive). It already
   converts a runtime Value to a WatAST. STUDY ITS SHAPE —
   our three primitives mirror this pattern (read a Value
   or registry entry; build a WatAST; return it as
   `Value::wat__WatAST` or `Value::holon__HolonAST`).
5. **`src/check.rs:11200-11230`** — the foldl/foldr
   TypeScheme registration. Note the `TypeScheme` shape:
   `type_params: Vec<String>`, `params: Vec<TypeExpr>`,
   `ret: TypeExpr`. NO param names — must synthesize.
6. **`src/check.rs:1149-1150`** — `register(name, scheme)`
   inserts into `self.schemes: HashMap<String, TypeScheme>`.
   The substrate-primitive lookup path.
7. **`src/runtime.rs:2406-2410`** — the runtime dispatch
   match for `quote`, `quasiquote`, `struct->form`. Your
   three new primitives register dispatch arms here (or
   adjacent).
8. **`src/runtime.rs:5736+` + `5757+`** — quote +
   quasiquote impls for the WatAST construction patterns.

## What to build

### Three substrate primitives

All three return `Value::holon__HolonAST(Arc<HolonAST>)` wrapped
as `:Option<wat::holon::HolonAST>`. Lookup precedence:

1. `Environment::lookup(name)` → if `Some(Value::wat__core__lambda(f))`,
   reconstruct from the `Function`
2. Else `env.schemes.get(name)` → if `Some(scheme)`, synthesize
   from the `TypeScheme`
3. Else return `:None`

#### `:wat::core::lookup-define`

Signature:
```
(:wat::core::lookup-define (name :wat::core::Symbol))
  -> :wat::core::Option<wat::holon::HolonAST>
```

Returns the FULL define AST: `(:wat::core::define <head> <body>)`.

For user defines, reconstruct from the `Function`:
- `<head>` = `(<name><type_params> (param[i] :param_types[i]) ... -> :ret_type)`
- `<body>` = the `body` field's `WatAST` directly

For substrate primitives, synthesize:
- `<head>` = `(<name><type_params> (_a0 :params[0]) (_a1 :params[1]) ... -> :ret)`
- `<body>` = sentinel `(:wat::core::__internal/primitive <name>)` — a
  marker indicating "Rust-implemented primitive; no wat-side body."

For unknown name, return `:None`.

#### `:wat::core::signature-of`

Signature:
```
(:wat::core::signature-of (name :wat::core::Symbol))
  -> :wat::core::Option<wat::holon::HolonAST>
```

Returns ONLY the head (signature) AST:
`(<name><type_params> (param :Type) ... -> :Ret)`.

Same lookup precedence as `lookup-define`. For both user
defines and substrate primitives, returns the head only
(equivalent to extracting the second element of what
`lookup-define` returns). For unknown name, return `:None`.

#### `:wat::core::body-of`

Signature:
```
(:wat::core::body-of (name :wat::core::Symbol))
  -> :wat::core::Option<wat::holon::HolonAST>
```

Returns the body AST.

For user defines, returns the `body` field's `WatAST`
directly (wrapped as HolonAST).

For substrate primitives, returns `:None` (no wat-side body
exists; the sentinel from `lookup-define` is for the FULL
define structure, not for `body-of` standalone).

For unknown name, return `:None`.

### Helper functions (Rust-internal; not exposed to wat)

#### `fn function_to_define_ast(f: &Function) -> WatAST`

Reconstructs the full `(:wat::core::define <head> <body>)`
AST from a stored `Function`. Used by `eval_lookup_define`
on the user-define path.

```rust
fn function_to_define_ast(f: &Function) -> WatAST {
    let head = function_to_signature_ast(f);
    let body = (*f.body).clone();
    WatAST::List(vec![
        WatAST::Keyword(":wat::core::define".into(), Span::unknown()),
        head,
        body,
    ], Span::unknown())
}
```

#### `fn function_to_signature_ast(f: &Function) -> WatAST`

Reconstructs the head: `(name<type_params> (param :Type) ... -> :Ret)`.

#### `fn type_scheme_to_signature_ast(name: &str, scheme: &TypeScheme) -> WatAST`

For a substrate primitive — synthesize the head from the
TypeScheme. Param names: `:_a0`, `:_a1`, ..., `:_a<n-1>`.

#### `fn primitive_to_define_ast(name: &str, scheme: &TypeScheme) -> WatAST`

For a substrate primitive — full `(:wat::core::define <head>
<sentinel-body>)`. Used by `eval_lookup_define` on the
substrate-primitive path.

### Registration

Each of the three primitives registers in two places:

1. **Runtime dispatch** (`src/runtime.rs` near line 2406):
   ```rust
   ":wat::core::lookup-define" => eval_lookup_define(args, env, sym),
   ":wat::core::signature-of" => eval_signature_of(args, env, sym),
   ":wat::core::body-of" => eval_body_of(args, env, sym),
   ```

2. **Type scheme** (`src/check.rs` — register alongside
   existing `:wat::core::*` schemes):
   ```rust
   env.register(":wat::core::lookup-define".into(), TypeScheme {
       type_params: vec![],
       params: vec![type_keyword("wat::core::Symbol")],
       ret: option_of(type_keyword("wat::holon::HolonAST")),
   });
   // similar for signature-of and body-of
   ```

(Adjust the helper functions for `type_keyword` / `option_of`
to match what's already used in check.rs for similar registrations
— see how `Option<T>` schemes are built around the existing
primitives.)

## Constraints

- **TWO Rust files modify:** `src/runtime.rs` (eval funcs +
  dispatch arms + helpers) + `src/check.rs` (scheme
  registrations). No wat files. No tests/* Rust files. No
  other crate. No documentation.
- **Workspace stays GREEN:** `cargo test --release --workspace`
  exits 0 after your changes. The new primitives don't break
  any existing tests.
- **No commits, no pushes.**
- **Match existing style:** the runtime's `eval_*` functions
  follow a consistent pattern (arg validation → execute →
  return Value). Mirror `eval_struct_to_form` (`runtime.rs:5891+`)
  as the closest precedent for "Value → WatAST" conversion.
  The check.rs registrations are simple two-line additions.
- **Use `Span::unknown()` for synthesized AST nodes** — there's
  no source span for primitives. (For user-define
  reconstruction, you can either use `Span::unknown()` or
  copy the `body`'s span — either is fine; consistent is
  better.)
- **Sentinel form for primitive bodies:** use
  `(:wat::core::__internal/primitive <name>)` as a literal
  marker. It's never evaluated — the substrate primitives
  use Rust dispatch — but `lookup-define` needs SOMETHING in
  the body slot. `body-of` returns `:None` for primitives
  rather than this sentinel.

## Tests (add to existing `tests/wat_*.rs` or wat-tests/)

For each primitive, three test cases:

1. **User-define lookup** — define a wat function, call the
   primitive, assert the returned AST matches the expected
   shape.
2. **Substrate-primitive lookup** — call the primitive on
   `:wat::core::foldl` (or any other registered primitive),
   assert the returned AST has the expected synthesized
   shape (`:_a0`, `:_a1`, etc.).
3. **Unknown name** — call the primitive on a non-existent
   name, assert returns `:None`.

For `body-of` specifically, also add a test that confirms
substrate primitives return `:None` (not the sentinel).

Total: 9-12 tests across the three primitives. Place them
in a new wat test file (e.g., `wat-tests/lookup.wat` or
similar) OR a new Rust integration test
(`tests/wat_lookup.rs`). Pick whichever matches existing
patterns most cleanly.

## What success looks like

1. `cargo test --release --workspace`: exit=0; all existing
   tests pass; 9-12 new tests pass for the three primitives.
2. The three primitives dispatch in runtime.rs + register in
   check.rs.
3. The 4 helper Rust functions (`function_to_define_ast`,
   `function_to_signature_ast`, `type_scheme_to_signature_ast`,
   `primitive_to_define_ast`) are present.
4. From a wat REPL or test, `(:wat::core::signature-of
   :wat::core::foldl)` returns the synthesized head AST.

## Reporting back

Target ~250 words:

1. **The three eval functions** — name + line range in
   runtime.rs.
2. **The 4 helper functions** — names + line ranges.
3. **The dispatch arms + scheme registrations** — line
   numbers.
4. **The synthesized AST shape** for `signature-of
   :wat::core::foldl` — quote it verbatim from a test
   assertion. Should look like:
   ```
   (:wat::core::foldl<T,Acc>
     (_a0 :wat::core::Vec<T>)
     (_a1 :Acc)
     (_a2 :wat::core::fn(Acc,T)->Acc)
     -> :Acc)
   ```
   (or the closest readable approximation given how
   TypeExpr renders).
5. **Test totals** — `cargo test --release --workspace`
   passed/failed/ignored; 9-12 new tests' names.
6. **Honest deltas** — anything you needed to invent or
   adapt because the brief's spec didn't directly transcribe
   (e.g., the existing `option_of` helper in check.rs has a
   different name).
7. **LOC delta** — runtime.rs + check.rs additions.

## Sequencing — what to do, in order

1. Read DESIGN.md cover to cover (especially Findings + Slices
   + Resolution-order).
2. Read `runtime.rs:499-510` (Function struct), `:563`
   (lookup), `:5891+` (struct->form precedent).
3. Read `check.rs:11200-11230` (foldl scheme registration),
   `:1149` (register API).
4. Implement the 4 helper Rust functions (in runtime.rs near
   the other Value→AST helpers).
5. Implement `eval_lookup_define`, `eval_signature_of`,
   `eval_body_of` (mirroring `eval_struct_to_form`'s arg
   validation pattern).
6. Add the 3 dispatch arms in runtime.rs.
7. Add the 3 scheme registrations in check.rs.
8. Run `cargo test --release --workspace` to confirm baseline
   still passes.
9. Add 9-12 new tests covering user-define lookup, substrate-
   primitive lookup, unknown-name return-`:None`, body-of
   substrate-primitive returns `:None`.
10. Run `cargo test --release --workspace` again.
11. Report per "Reporting back."

Then DO NOT commit. Working tree stays modified for the
orchestrator to score.

## Why this slice matters for the chain

Slice 1 is the ONLY Rust work in arc 143. After it ships,
slices 2-5 are pure wat (helpers + defmacro + apply +
closure). The substrate gains the introspection bridge it's
been missing since macros shipped — every future userland
macro that needs to ASK about an existing binding gets these
three primitives for free.

This slice also exercises the failure-engineering discipline
in a fresh shape — the prior arc 130 RELAND surfaced the
reduce gap; arc 143 closes it via a substrate addition;
arc 130 then unblocks. The chain is: gap surfaces (RELAND),
substrate fix ships (arc 143), original work resumes (arc
130 slice 1 RELAND v2).
