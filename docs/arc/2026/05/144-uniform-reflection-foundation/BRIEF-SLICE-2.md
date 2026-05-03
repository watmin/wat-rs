# Arc 144 Slice 2 — Sonnet Brief — Special-form registry

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/check.rs:2950-3160` (the `infer_list` head dispatch — primary
special-form dispatch), `src/check.rs:1420` + `1437` + `1461`
(match / Result/expect / Option/expect dispatch sites),
`src/check.rs:1524-1527`, `2098-2101`, `2463-2466`
(sandboxed-exec dispatch sites), `src/check.rs:1393-1396`,
`2537-2542` (channel-op dispatch), `src/freeze.rs:831-836`
(definitional special forms — define/defmacro/struct/enum/newtype/
typealias), `src/runtime.rs:2400-2425` (lambda / quasiquote /
match runtime dispatch). Confirmed slice 1's `lookup_form`
SpecialForm path returns None today; `Binding::SpecialForm
{ name, signature: HolonAST, doc_string }` is in place to receive
populated registry results.

**Goal:** ship a special-form registry covering ~25-30 special forms
identified from the dispatch crawl. Each form gets a name + a
synthetic HolonAST signature sketch + `None` doc_string. Slice 1's
`lookup_form` consults the registry on its 5th branch; the existing
`Binding::SpecialForm` arm in the 3 reflection primitives just works
because the dispatch is already in place.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/144-uniform-reflection-foundation/DESIGN.md`**
   — slice 2 description (special-form registrations).
2. **`docs/arc/2026/05/144-uniform-reflection-foundation/SCORE-SLICE-1.md`**
   — what slice 1 shipped (Binding enum, lookup_form's 4-registry
   walk, the SpecialForm branch that returns None, the 3 reflection
   primitives' existing SpecialForm dispatch arm).
3. **`src/runtime.rs:6267+`** — slice 1's `Binding<'a>::SpecialForm
   { name, signature: HolonAST, doc_string }`. Slice 2 populates the
   `signature` field via the registry.
4. **`src/runtime.rs:6315+`** — slice 1's `lookup_form`. Slice 2
   adds the 5th branch (between Type and `None`) consulting the
   special-form registry.
5. **`src/runtime.rs:6420-6500`** — the 3 reflection primitives'
   `Binding::SpecialForm` dispatch arms (lookup-define emits the
   `(:wat::core::__internal/special-form <name>)` sentinel;
   signature-of returns the stored `signature` field directly;
   body-of returns `:None`). NO CHANGES needed here — the dispatch
   arms already handle SpecialForm.
6. **`src/check.rs:2950-3160`** — primary special-form dispatch in
   `infer_list`. Audit ALL the explicit `":wat::core::*"` /
   `":wat::kernel::*"` heads dispatched here.
7. **`src/freeze.rs:825-840`** — definitional special forms
   handled at freeze (top-level only).

## What to ship

### 1. NEW module `src/special_forms.rs`

Standalone module housing the registry + lookup API. Add
`pub mod special_forms;` to `src/lib.rs` (next to other `pub mod`
declarations).

```rust
//! Arc 144 slice 2 — special-form registry.
//!
//! Special forms are syntactic constructs the type checker + runtime
//! handle directly (not via dispatch through Function or TypeScheme).
//! Examples: `:wat::core::if`, `let*`, `lambda`, `define`, `match`,
//! `quasiquote`, `try`, the spawn family, the channel ops.
//!
//! This registry lets `:wat::runtime::lookup-form` (arc 144 slice 1)
//! return `Binding::SpecialForm` for each known form, exposing a
//! synthesized signature sketch the consumer (e.g., a future `(help
//! :if)` form) can render.
//!
//! Each entry carries the form's full keyword name + a synthesized
//! HolonAST showing the syntax shape + a placeholder `None`
//! doc_string (arc 141 will populate it).

use crate::ast::HolonAST;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct SpecialFormDef {
    pub name: String,
    pub signature: HolonAST,
    pub doc_string: Option<String>,
}

static REGISTRY: OnceLock<HashMap<String, SpecialFormDef>> = OnceLock::new();

pub fn lookup_special_form(name: &str) -> Option<&'static SpecialFormDef> {
    REGISTRY.get_or_init(build_registry).get(name)
}

fn build_registry() -> HashMap<String, SpecialFormDef> {
    let mut m = HashMap::new();
    // ... ~25-30 special-form registrations here
    m
}
```

### 2. Special-form enumeration + signature sketch FORMAT

Each signature sketch is a `HolonAST::Bundle` whose first child is
the form's head as a Keyword; remaining children are bare-symbol
placeholders for the syntactic slots. Repeating slots use `<name>+`
(one or more) or `<name>*` (zero or more). Nested grammar uses
nested Bundles.

#### Sketch format examples

```
:wat::core::if
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::if"),
    HolonAST::symbol("<cond>"),
    HolonAST::symbol("<then>"),
    HolonAST::symbol("<else>"),
  ])

:wat::core::let*
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::let*"),
    HolonAST::symbol("<bindings>"),     ;; nested grammar; symbol placeholder
    HolonAST::symbol("<body>+"),
  ])

:wat::core::cond
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::cond"),
    HolonAST::symbol("<clause>+"),       ;; <clause> ::= (<test> <body>+)
  ])

:wat::core::lambda
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::lambda"),
    HolonAST::symbol("<params>"),        ;; (<arg-name>*)
    HolonAST::symbol("<body>+"),
  ])

:wat::core::match
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::match"),
    HolonAST::symbol("<scrutinee>"),
    HolonAST::symbol("->"),
    HolonAST::symbol("<T>"),
    HolonAST::symbol("<arm>+"),          ;; <arm> ::= (<pattern> <body>)
  ])

:wat::core::define
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::define"),
    HolonAST::symbol("<head>"),          ;; (name<T,...> (arg :T)... -> :Ret)
    HolonAST::symbol("<body>"),
  ])

:wat::core::struct
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::struct"),
    HolonAST::symbol("<name>"),          ;; :Type<T,...>
    HolonAST::symbol("<field>+"),        ;; (<name> :T)
  ])

:wat::core::quote
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::quote"),
    HolonAST::symbol("<expr>"),
  ])

:wat::core::option::expect (RETIRED)
  HolonAST::Bundle([
    HolonAST::keyword(":wat::core::option::expect"),
    HolonAST::symbol("<retired-use-Option/expect>"),
  ])

:wat::kernel::spawn-program-ast
  HolonAST::Bundle([
    HolonAST::keyword(":wat::kernel::spawn-program-ast"),
    HolonAST::symbol("<forms-block>"),   ;; (:wat::core::forms ...)
    HolonAST::symbol("<arg>*"),
  ])
```

Use `crate::ast::HolonAST::keyword(name)` and
`crate::ast::HolonAST::symbol(name)` constructors (they exist per
slice 5b's runtime work).

### 3. The 25-30 special forms to register

Audit `src/check.rs:2950-3160` + `src/freeze.rs:825-840` +
`src/runtime.rs:2400-2425` for the comprehensive list. The
orchestrator's pre-flight crawl identifies these (sonnet should
verify completeness):

#### Control / branching (5)
- `:wat::core::if` — `(:wat::core::if <cond> <then> <else>)`
- `:wat::core::cond` — `(:wat::core::cond <clause>+)`
- `:wat::core::match` — `(:wat::core::match <scrutinee> -> <T> <arm>+)`
- `:wat::core::let` — `(:wat::core::let <bindings> <body>+)`
- `:wat::core::let*` — `(:wat::core::let* <bindings> <body>+)`

#### Lambdas / functions (3)
- `:wat::core::lambda` — `(:wat::core::lambda <params> <body>+)`
- `:wat::core::define` — `(:wat::core::define <head> <body>)`
- `:wat::core::defmacro` — `(:wat::core::defmacro <head> <template>)`

#### Type definitions (4)
- `:wat::core::struct` — `(:wat::core::struct <name> <field>+)`
- `:wat::core::enum` — `(:wat::core::enum <name> <variant>+)`
- `:wat::core::newtype` — `(:wat::core::newtype <name> <target>)`
- `:wat::core::typealias` — `(:wat::core::typealias <name> <target>)`

#### Error handling (5 — including 3 retired-as-poison)
- `:wat::core::Result/try` — `(:wat::core::Result/try <expr>)`
- `:wat::core::Option/try` — `(:wat::core::Option/try <expr>)`
- `:wat::core::Option/expect` — `(:wat::core::Option/expect -> <T> <opt> <msg>)`
- `:wat::core::Result/expect` — `(:wat::core::Result/expect -> <T> <res> <msg>)`
- `:wat::core::try` (RETIRED → use `Result/try`) — sketch names the redirect
- `:wat::core::option::expect` (RETIRED → `Option/expect`)
- `:wat::core::result::expect` (RETIRED → `Result/expect`)

#### Quote / quasiquote / AST (5)
- `:wat::core::quote` — `(:wat::core::quote <expr>)`
- `:wat::core::quasiquote` — `(:wat::core::quasiquote <template>)`
- `:wat::core::unquote` — `(:wat::core::unquote <expr>)` — only valid inside quasiquote
- `:wat::core::unquote-splicing` — `(:wat::core::unquote-splicing <expr>)` — only valid inside quasiquote
- `:wat::core::forms` — `(:wat::core::forms <form>*)`
- `:wat::core::struct->form` — `(:wat::core::struct->form <struct-value>)`

#### Sandboxed exec / spawn (4-6)
- `:wat::kernel::spawn` — `(:wat::kernel::spawn <body>+)` (audit shape)
- `:wat::kernel::spawn-thread` — sketch per the shape in check.rs
- `:wat::kernel::spawn-program-ast` — `(:wat::kernel::spawn-program-ast <forms-block> <arg>*)`
- `:wat::kernel::run-sandboxed-ast` — analogous shape
- `:wat::kernel::run-sandboxed-hermetic-ast` — analogous shape
- `:wat::kernel::fork-program-ast` — analogous shape

#### Channel ops (5-6)
- `:wat::kernel::send` — `(:wat::kernel::send <tx> <value>)` — return type :Result<:Option<...>, :ThreadDiedError>
- `:wat::kernel::recv` — `(:wat::kernel::recv <rx>)` — same
- `:wat::kernel::try-recv` — `(:wat::kernel::try-recv <rx>)`
- `:wat::kernel::select` — `(:wat::kernel::select <arm>+)`
- `:wat::kernel::process-send` — analogous
- `:wat::kernel::process-recv` — analogous

**Note:** channel ops are dispatched via `infer_list` AND have an
arc-110 grammar restriction (only inside `match` or `option::expect`).
The signature sketch should reflect the surface form, not the
grammar restriction. Sonnet judges per case.

If your audit surfaces additional special forms not in this list,
ADD THEM with a comment naming the dispatch site you found them at.
If a form on this list isn't actually a special form (per check.rs's
dispatch), REMOVE it with a comment.

### 4. Wire `lookup_form` to consult the registry

In `src/runtime.rs`'s `lookup_form` (slice 1), add the 5th branch
between Type and the trailing `None`:

```rust
// 5. Special forms — slice 2 populated.
if let Some(def) = crate::special_forms::lookup_special_form(name) {
    return Some(Binding::SpecialForm {
        name: def.name.clone(),
        signature: def.signature.clone(),
        doc_string: def.doc_string.clone(),
    });
}
None
```

Cloning HolonAST per lookup is acceptable on the reflection-only
path. ~25-30 small ASTs total.

### 5. Tests

NEW `tests/wat_arc144_special_forms.rs` with 8+ tests:

1. **`lookup_form_if_returns_special_form`** — call lookup-define
   on `:wat::core::if`; assert returned AST contains
   `:wat::core::__internal/special-form`. Call signature-of; assert
   returned AST head is `:wat::core::if` + the placeholder slots
   are visible in the rendered output. Call body-of; assert :None.
2. **`lookup_form_let_star_returns_special_form`** — same shape for
   `:wat::core::let*`.
3. **`lookup_form_lambda_returns_special_form`** — same shape for
   `:wat::core::lambda`.
4. **`lookup_form_define_returns_special_form`** — same for
   `:wat::core::define`.
5. **`lookup_form_match_returns_special_form`** — same for
   `:wat::core::match`.
6. **`lookup_form_quasiquote_returns_special_form`** — same for
   `:wat::core::quasiquote`.
7. **`lookup_form_struct_returns_special_form`** — same for
   `:wat::core::struct`.
8. **`lookup_form_kernel_send_returns_special_form`** — same for
   `:wat::kernel::send`.
9. **(BONUS)** `lookup_form_unknown_special_form_name_returns_none`
   — `:wat::core::not-a-special-form` returns :None.

Tests follow `tests/wat_arc144_lookup_form.rs` (slice 1's) shape:
startup_from_source + invoke_user_main + stdout assertions.

### 6. Workspace verification

After the slice ships:

```
cargo test --release --test wat_arc144_special_forms    # new tests pass
cargo test --release --test wat_arc144_lookup_form      # slice 1 tests still pass
cargo test --release --test wat_arc143_lookup           # arc 143 lookup tests still pass
cargo test --release --test wat_arc143_manipulation     # 8/8 still pass
cargo test --release --test wat_arc143_define_alias     # 2/3 (length canary unchanged — slice 3 territory)
cargo test --release --workspace                         # baseline failure profile
```

Same baseline failure profile as today: only the slice 6 length
canary fails. ZERO new regressions.

```
cargo clippy --release --all-targets
```

No new warnings.

## Constraints

- **NEW file:** `src/special_forms.rs`. Add `pub mod special_forms;`
  to `src/lib.rs`.
- **Edit `src/runtime.rs`:** add ~6-line registry-consult branch in
  `lookup_form`. NO other changes to runtime.rs (the 3 reflection
  primitives' SpecialForm dispatch arms already work).
- **NEW test file:** `tests/wat_arc144_special_forms.rs`.
- **No edits to `src/check.rs` or `src/macros.rs` or `src/freeze.rs`.**
  This slice is purely additive to the reflection layer; check
  dispatch is untouched.
- **No commits, no pushes.**

## What success looks like

1. `src/special_forms.rs` exists with `SpecialFormDef` struct,
   static OnceLock-backed registry, `lookup_special_form` API,
   and ~25-30 form registrations (audit confirms which).
2. `lookup_form` in `src/runtime.rs` has a 5th branch consulting
   the registry.
3. New `tests/wat_arc144_special_forms.rs` with 8+ tests; ALL pass.
4. Slice 1's tests (`tests/wat_arc144_lookup_form.rs`) ALL still pass.
5. Arc 143 tests unchanged: `wat_arc143_lookup` 11/11,
   `wat_arc143_manipulation` 8/8, `wat_arc143_define_alias` 2/3.
6. `cargo test --release --workspace`: same baseline failure profile.
7. `cargo clippy --release --all-targets`: no new warnings.

## Reporting back

Target ~250-350 words.

1. **`SpecialFormDef` + registry shape** — quote the struct + the
   API signatures.
2. **The form enumeration** — count + grouping (control / lambda /
   typedef / error / quote / spawn / channel). If you DEVIATED from
   the brief's count (added or removed forms), name each delta + the
   dispatch site evidence.
3. **A representative sketch** — quote ONE sketch's HolonAST
   construction verbatim to show the format you used.
4. **The lookup_form integration** — quote the new branch verbatim.
5. **Test totals** — the 8+ new tests pass; arc 143/144 baseline
   tests unchanged; workspace failure profile is the same as
   pre-slice-2.
6. **clippy** — quote any warnings (expected: none).
7. **Honest deltas** — anything you needed to investigate / adapt.

## Sequencing

1. Read pre-reads in order.
2. Audit `src/check.rs:2950-3160` + `src/freeze.rs:825-840` +
   `src/runtime.rs:2400-2425` for the comprehensive special-form
   list. Cross-check against the brief's enumeration; deltas →
   add to your registry with comments naming the dispatch site.
3. Create `src/special_forms.rs` with the struct + registry +
   `lookup_special_form` API + `build_registry` populating ALL
   audited forms.
4. Add `pub mod special_forms;` to `src/lib.rs`.
5. Add the 5th branch to `lookup_form` in `src/runtime.rs`.
6. Create `tests/wat_arc144_special_forms.rs` with 8+ tests.
7. Run `cargo test --release --test wat_arc144_special_forms`
   first — confirm new tests pass.
8. Run the slice 1 + arc 143 baseline tests — confirm no
   regressions.
9. Run `cargo test --release --workspace` — confirm baseline
   failure profile.
10. Run `cargo clippy --release --all-targets` — confirm clean.
11. Report.

Then DO NOT commit. Working tree stays modified for orchestrator to
score.

## Why this slice matters

Slice 2 closes the SpecialForm coverage gap that slice 1 opened.
After this slice, the user's principle "nothing is special — `(help
:if) /just works/`" holds for all known special forms in the
substrate.

Slice 3 (TypeScheme registrations for the 15 hardcoded callable
primitives) is independent — runs in parallel or after this slice.
Slice 4 (verification including the slice 6 length canary turning
green) blocks on slice 3.

The static OnceLock-backed registry is per ZERO-MUTEX doctrine:
atomics + OnceLock are the permitted concurrency primitives;
HashMap inside OnceLock is initialized once on first access and
read-shared forever. No Mutex; no RwLock.
