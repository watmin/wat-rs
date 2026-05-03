# Arc 144 Slice 1 — Sonnet Brief — Binding enum + lookup_form refactor

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/runtime.rs:6088-6271` (the arc 143 slice 1 reflection primitives
+ helpers + LookupResult enum + lookup_callable), `src/runtime.rs:498`
(Function struct), `src/runtime.rs:678` (SymbolTable struct including
.functions / .macro_registry / .types fields), `src/macros.rs:55-95`
(MacroDef + MacroRegistry::get), `src/types.rs:120-180` (TypeDef +
TypeEnv::get), `src/check.rs:60-67` (TypeScheme), `src/check.rs:1070-1117`
(CheckEnv + with_builtins), `src/runtime.rs:2410-2417` (dispatch arms),
`src/check.rs:3120-3204` (special-case branches in infer_list), and
`tests/wat_arc143_lookup.rs` (slice 1 test pattern).

**Goal:** ship the gating substrate change — define a uniform
`Binding` enum (5 variants), refactor `lookup_callable` →
`lookup_form` returning `Option<Binding>` walking the 4 existing
registries (SpecialForm registry comes in slice 2), and refactor the
3 arc 143 slice 1 reflection primitives (`lookup-define`,
`signature-of`, `body-of`) to dispatch on Binding while preserving
their current behavior for UserFunction + Primitive AND extending
emission to Macro + Type variants.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/144-uniform-reflection-foundation/DESIGN.md`**
   — the full arc design, especially the Binding enum sketch + the
   per-variant accessor list.
2. **`docs/arc/2026/05/143-define-alias/INSCRIPTION.md`** — what
   shipped in arc 143 (the lookup_callable + LookupResult enum +
   eval_lookup_define / signature_of / body_of are arc 143's slice 1
   work; this slice refactors them).
3. **`src/runtime.rs:6088-6271`** — the existing reflection
   primitives + LookupResult + lookup_callable + helper functions.
   This slice REPLACES `LookupResult` with `Binding` and renames
   `lookup_callable` to `lookup_form`.
4. **`src/macros.rs:55-95`** — `MacroDef` (name, params, rest_param,
   body, span) + `MacroRegistry::get(name) -> Option<&MacroDef>`.
5. **`src/types.rs:120-180`** — `TypeDef` enum (Struct / Enum /
   Newtype / Alias) + `TypeDef::name(&self) -> &str` accessor +
   `TypeEnv::get(name) -> Option<&TypeDef>`.
6. **`src/runtime.rs:678-725`** — `SymbolTable` fields:
   `.functions`, `.macro_registry: Option<Arc<MacroRegistry>>`,
   `.types: Option<Arc<TypeEnv>>`.
7. **`src/runtime.rs:5970-6094`** — the existing emission helpers
   (`function_to_signature_ast`, `function_to_define_ast`,
   `type_scheme_to_signature_ast`, `primitive_to_define_ast`,
   `name_from_keyword_or_lambda`). This slice ADDS sibling helpers
   for Macro + Type variants.
8. **`tests/wat_arc143_lookup.rs`** — slice 1 test pattern (run
   wat source via `startup_from_source` + `invoke_user_main`,
   capture stdout, assert lines). New tests follow this pattern.

## What to ship

### 1. The `Binding` enum

NEW enum next to `LookupResult` in `src/runtime.rs` (around line
6116). The five variants per arc 144 DESIGN:

```rust
/// Arc 144 slice 1 — uniform reflection binding. Every kind of
/// known wat form (user defines, macros, substrate primitives,
/// special forms, types) produces a Binding when looked up.
///
/// Each variant carries a `doc_string: Option<String>` field as the
/// paved road for arc 141 (docstrings). Always `None` until arc 141
/// populates it.
pub enum Binding<'a> {
    UserFunction {
        name: String,
        f: &'a Arc<Function>,
        doc_string: Option<String>,
    },
    Macro {
        name: String,
        def: &'a MacroDef,
        doc_string: Option<String>,
    },
    Primitive {
        name: String,
        scheme: TypeScheme,
        doc_string: Option<String>,
    },
    SpecialForm {
        name: String,
        // Slice 2 populates this with synthetic signature ASTs +
        // doc_string at registration time. Slice 1 carries the shape.
        signature: HolonAST,
        doc_string: Option<String>,
    },
    Type {
        name: String,
        def: &'a TypeDef,
        doc_string: Option<String>,
    },
}
```

The `'a` lifetime parameter ties UserFunction / Macro / Type to the
SymbolTable's borrowed data (existing pattern from `LookupResult`).
Primitive / SpecialForm own their data (TypeScheme is owned in
LookupResult today; HolonAST is owned construction).

**Use the existing imports.** Reuse `Function`, `MacroDef`,
`TypeDef`, `TypeScheme` — don't re-declare. `HolonAST` is in
`crate::ast` (sibling import already exists in runtime.rs for the
arc 143 slice 3 work).

**`pub` visibility.** The enum + variants need to be reachable from
test code (the new tests will pattern-match on it).

### 2. Refactor `lookup_callable` → `lookup_form`

REPLACE `lookup_callable` (runtime.rs:6102-6114) with `lookup_form`
returning `Option<Binding>`. Lookup precedence (mirrors call
dispatch):

```rust
pub fn lookup_form<'a>(
    name: &str,
    sym: &'a SymbolTable,
) -> Option<Binding<'a>> {
    // 1. User defines (shadow builtins per call-dispatch precedent).
    if let Some(f) = sym.functions.get(name) {
        return Some(Binding::UserFunction {
            name: name.to_string(),
            f,
            doc_string: None,
        });
    }
    // 2. Macros — only if the SymbolTable has a registry attached.
    if let Some(reg) = &sym.macro_registry {
        if let Some(def) = reg.get(name) {
            return Some(Binding::Macro {
                name: name.to_string(),
                def,
                doc_string: None,
            });
        }
    }
    // 3. Substrate primitives via on-demand CheckEnv.
    let env = crate::check::CheckEnv::with_builtins();
    if let Some(scheme) = env.get(name) {
        return Some(Binding::Primitive {
            name: name.to_string(),
            scheme: scheme.clone(),
            doc_string: None,
        });
    }
    // 4. Types — only if the SymbolTable has a type registry attached.
    if let Some(types) = &sym.types {
        if let Some(def) = types.get(name) {
            return Some(Binding::Type {
                name: name.to_string(),
                def,
                doc_string: None,
            });
        }
    }
    // 5. SpecialForm registry — slice 2 populates. Returns None today.
    None
}
```

DELETE `LookupResult` (it's replaced; no public consumers per grep
verification — sonnet should grep `LookupResult` to confirm no
external consumers before deleting; the enum was added in arc 143
slice 1 and only used in arc 143 slice 1's three eval_* primitives).

### 3. Refactor the 3 reflection primitives to dispatch on Binding

For each of `eval_lookup_define`, `eval_signature_of`, `eval_body_of`
(runtime.rs:6132-6271): replace the `match lookup_callable(...)`
arms with `match lookup_form(...)` arms covering ALL five Binding
variants. Behavior per variant:

#### `eval_lookup_define` — emits the FULL form (define / defmacro / type-decl)

| Variant | Emission |
|---|---|
| UserFunction | `(:wat::core::define <head> <body>)` — existing `function_to_define_ast(f)` behavior. |
| Primitive | `(:wat::core::define <head> <sentinel>)` — existing `primitive_to_define_ast(name, scheme)` behavior. |
| Macro | `(:wat::core::defmacro <head> <template>)` — NEW helper `macrodef_to_define_ast(def)`. The HEAD is `(name (param :AST<T>)... -> :AST<R>)`; the substrate doesn't preserve the param `:AST<T>` types separately from the template, so synthesize each param's annotation as `:AST<wat::WatAST>` (honest sentinel — the param is an AST, but the specific T isn't tracked). The TEMPLATE is the stored `def.body` WatAST verbatim. |
| Type | The type's declaration form: `(:wat::core::struct ...)` / `(:wat::core::enum ...)` / `(:wat::core::newtype ...)` / `(:wat::core::typealias ...)` — NEW helper `typedef_to_define_ast(def)`. Slice 1 emits a MINIMAL HEAD-ONLY shape: e.g., for a struct, `(:wat::core::struct :MyType<T> ...)` where `...` is a literal `(:wat::core::__internal/struct-fields :MyType)` sentinel. Real field emission is deferred to a future arc — slice 1's contract is "the form names the right declaration head + the type's name; readers know to grep for the actual decl." Honest sentinel beats a half-rendered struct. |
| SpecialForm | `(:wat::core::__internal/special-form <name>)` sentinel — slice 2 will populate the registry; until then this arm is unreachable. Add the dispatch arm with a sentinel emission so the code is structurally complete. |

#### `eval_signature_of` — emits the SIGNATURE HEAD only

| Variant | Emission |
|---|---|
| UserFunction | existing `function_to_signature_ast(f)` behavior. |
| Primitive | existing `type_scheme_to_signature_ast(name, scheme)` behavior. |
| Macro | NEW helper `macrodef_to_signature_ast(def)` — emits `(name (p :AST<wat::WatAST>) ... -> :AST<wat::WatAST>)`. Same honest sentinel rationale as define-ast. |
| Type | NEW helper `typedef_to_signature_ast(def)` — emits the head ONLY: `(:MyType<T>)` where `<T>` is the type-param suffix. For all four TypeDef variants, this is just the FQDN keyword + parametric suffix. No body, no field info — that's `body-of`'s territory (and slice 1 returns :None there for types). |
| SpecialForm | The Binding's `signature` field directly (it's already a HolonAST). Slice 2 populates it. |

#### `eval_body_of` — emits the BODY when one exists

| Variant | Emission |
|---|---|
| UserFunction | existing behavior — body field WatAST → HolonAST. |
| Primitive | existing behavior — :None (Rust-implemented, no wat body). |
| Macro | NEW — the macro's TEMPLATE is its body. Emit `def.body` WatAST → HolonAST. |
| Type | :None (types have declarations, not bodies — declarations are the lookup-define output). |
| SpecialForm | :None (special forms are semantic operations, not data with a body). |

### 4. Helper additions (~40-100 LOC across 4 helpers)

NEW helpers in src/runtime.rs near the existing helpers (5970-6082):

- `fn macrodef_to_define_ast(def: &MacroDef) -> WatAST`
- `fn macrodef_to_signature_ast(def: &MacroDef) -> WatAST`
- `fn typedef_to_define_ast(def: &TypeDef) -> WatAST`
- `fn typedef_to_signature_ast(def: &TypeDef) -> WatAST`

Each is small (~10-30 LOC) and follows the existing helper pattern
(use Span::unknown(); construct WatAST::Keyword + WatAST::List).

For the Macro signature head, build:
```
(name (p1 :AST<wat::WatAST>) (p2 :AST<wat::WatAST>) ... -> :AST<wat::WatAST>)
```
Optionally append the rest_param if present:
```
... & (rest :AST<Vec<wat::WatAST>>) ...
```
The `&` separator is a `WatAST::Symbol` per `MacroDef::rest_param`'s
declaration syntax.

For the Type signature head, build the bare name + parametric
suffix per the TypeDef variant's name + type_params. Simplest for
slice 1: `(<name><type-params>)` as a single-element List with a
single Keyword head. (Type heads don't carry params/return — they
ARE just the name shape.)

### 5. Tests

NEW test file `tests/wat_arc144_lookup_form.rs` (mirrors
`tests/wat_arc143_lookup.rs` shape — startup_from_source +
invoke_user_main + stdout assertion).

5 tests minimum:

1. **Macro lookup** — define a macro via `:wat::core::defmacro`,
   call `:wat::runtime::lookup-define` on its name, assert the
   returned Option is Some + the AST contains
   `:wat::core::defmacro`. Ditto signature-of (assert head present)
   + body-of (assert template present).
2. **Type lookup** — define a struct, call `lookup-define`, assert
   Some + AST contains `:wat::core::struct` (or appropriate
   declaration head). Ditto signature-of (head only).
   `body-of` returns `:None` (verify).
3. **User-function lookup** — same shape as arc 143's existing
   test but verify it still works post-refactor (no regression).
4. **Substrate-primitive lookup** — same shape as arc 143's
   existing test for `:wat::core::foldl` (no regression).
5. **Unknown name** — call all three primitives on
   `:no::such::thing`; all return `:None`.

Plus 2-3 unit tests in src/runtime.rs::tests if the existing
pattern covers eval_* internals — sonnet should follow whichever
test convention the file uses.

### 6. Workspace verification

After the refactor, the existing tests must still pass:

```
cargo test --release --test wat_arc143_lookup
cargo test --release --test wat_arc143_manipulation
cargo test --release --test wat_arc143_define_alias
```

The first two should be FULLY GREEN (no behavior change for arc 143's
existing reflection primitives). The third stays at 2/3 (the length
canary is arc 144's slice 4 territory; this slice doesn't fix it).

```
cargo test --release --workspace
```

Should show the SAME failure profile as today: only the slice 6
length canary fails. ZERO new regressions.

## Constraints

- **2-3 Rust files modified:** `src/runtime.rs` (the core refactor) +
  `src/check.rs` (no changes expected — special-case branches at
  3126-3204 already accept arbitrary keyword args; no scheme update
  needed because lookup_form's runtime behavior is identical to
  lookup_callable's). Sonnet may discover the special-case names
  need updating for `:wat::runtime::lookup-form` in slice 4 — that's
  fine but NOT required here (the 3 existing primitives keep their
  names; lookup-form is added in slice 4 if needed).
- **NEW:** `tests/wat_arc144_lookup_form.rs`.
- **No wat files.** No new defines or macros in `wat/runtime.wat`
  (slice 4 may add a `lookup-form` wat-side wrapper if useful;
  not in scope here).
- **Workspace must compile + only the slice 6 length canary fails.**
- **No commits, no pushes.** Working tree stays modified for
  orchestrator to score.
- **Run `cargo clippy --release --all-targets` to verify no new warnings**
  (especially around the lifetime annotation on `Binding<'a>` and
  the `'a` propagation through `lookup_form`'s signature).

## What success looks like

1. `Binding` enum exists with all 5 variants + doc_string field.
2. `lookup_form` walks 4 registries; SpecialForm path returns None.
3. Three reflection primitives dispatch on Binding; UserFunction +
   Primitive behavior preserved exactly; Macro + Type emit
   appropriately; SpecialForm path is structurally present but
   unreachable.
4. New helpers `macrodef_to_define_ast` + `macrodef_to_signature_ast`
   + `typedef_to_define_ast` + `typedef_to_signature_ast`.
5. `LookupResult` enum DELETED.
6. New `tests/wat_arc144_lookup_form.rs` with 5 tests covering the
   added kinds + no-regression for existing kinds.
7. `cargo test --release --workspace` shows the same failure profile
   as pre-slice-1 (only slice 6 length canary).
8. `cargo clippy --release --all-targets` shows no new warnings.

## Reporting back

Target ~250-350 words.

1. **The Binding enum** — quote the variant declarations verbatim.
2. **The lookup_form signature + body** — quote verbatim or
   paraphrase the registry-walk order.
3. **The three primitive dispatch updates** — for each, list the 5
   match arms + their emission helper / sentinel.
4. **The four helper signatures** + line numbers.
5. **The 5 (or more) new tests** + their assertions in 1 sentence
   each.
6. **Workspace test totals** — confirm only slice 6 length canary
   fails; quote the cargo test summary line.
7. **clippy** — quote any warnings (expected: none).
8. **Honest deltas** — anything you needed to investigate / adapt
   beyond the brief.

## Sequencing

1. Read the required pre-reads in order.
2. **Grep `LookupResult` workspace-wide** to confirm no external
   consumers beyond `lookup_callable` and the 3 eval_* primitives.
   If any unexpected consumers exist, STOP and report.
3. Define the Binding enum.
4. Implement lookup_form (registry walk).
5. Add the 4 new emission helpers.
6. Refactor each of the 3 eval_* primitives to dispatch on Binding.
7. Delete LookupResult + lookup_callable.
8. Add the 5+ new tests in `tests/wat_arc144_lookup_form.rs`.
9. Run `cargo test --release --workspace` — confirm only the slice
   6 length canary fails; ZERO new regressions.
10. Run `cargo clippy --release --all-targets` — confirm no new
    warnings.
11. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

Slice 1 is the GATING substrate change for arc 144. Every later
slice (2 = special-form registry, 3 = hardcoded primitive
schemes, 4 = verification including the slice 6 length canary
turning green) builds on lookup_form returning a uniform Binding.
Once shipped, the SpecialForm + hardcoded-Primitive coverage
becomes additive — populate the relevant registries; lookup_form
+ the 3 eval_* primitives Just Work for those kinds because the
dispatch is already in place.

The user's principle — **"nothing is special — `(help :if) /just
works/`"** — manifests in this slice's enum design. Every form
kind sits in the same union; the consumer doesn't case-by-case
the kinds it cares about. The data flows uniformly through one
shape.
