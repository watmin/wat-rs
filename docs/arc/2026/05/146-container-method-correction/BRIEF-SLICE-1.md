# Arc 146 Slice 1 — Sonnet Brief — Substrate multimethod mechanism

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/macros.rs:306-348` (defmacro parsing pattern — template for
defmultimethod), `src/macros.rs:55-95` (MacroDef + MacroRegistry —
template for Multimethod + MultimethodRegistry), `src/runtime.rs:690`
(SymbolTable.macro_registry — capability-carrier pattern),
`src/runtime.rs:785` (set_macro_registry setter pattern),
`src/freeze.rs:828-841` (is_mutation_form — the top-level
construct list defmultimethod joins), `src/check.rs:2956+`
(infer_list head dispatch — where multimethod-routing inserts),
`src/check.rs:8323` (unify — used for arm-pattern matching),
`src/runtime.rs:2400+` (eval list-call dispatch — where runtime
multimethod routing inserts), `src/runtime.rs:6267` (Binding enum
gets a 6th variant), `src/runtime.rs:6315` (lookup_form gets a
6th branch), `src/special_forms.rs` (defmultimethod gets registered
as a special form per arc 144 slice 2's registry).

FM 9 baseline confirmed: `wat_arc144_lookup_form` 9/9,
`wat_arc144_special_forms` 9/9, `wat_arc144_hardcoded_primitives`
17/17, `wat_arc143_lookup` 11/11, `wat_arc143_define_alias` 2/3
(length canary still red — arc 146 slice 2 closes it).

**Goal:** ship the substrate multimethod MECHANISM. NO migration
of any existing primitive. After this slice, you can declare a
NEW multimethod in wat over arbitrary types and it dispatches +
type-checks + reflects correctly. Slice 2 then uses the mechanism
to migrate `:length` (and the slice 6 length canary turns green
as the proof).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/146-container-method-correction/DESIGN.md`**
   — full arc design, multimethod-as-mechanism framing,
   architectural decisions (pass-through arms; entity-kind not
   type-system-feature; cross-language reference).
2. **`docs/arc/2026/05/144-uniform-reflection-foundation/REALIZATIONS.md`**
   — the discovery cascade. ESPECIALLY Realization 6 (the
   discipline lesson) and the reordering after slice 3b
   cancellation.
3. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** § FM 10 + § 12 —
   the discipline + strategic context. This slice IS foundation
   work; velocity is the wrong currency.
4. **`src/macros.rs:55-95`** — `MacroDef` + `MacroRegistry` shape.
   You will mirror this for `Multimethod` + `MultimethodRegistry`.
5. **`src/macros.rs:306-348`** — `is_defmacro_form` +
   `parse_defmacro_form`. You will mirror this for
   `is_defmultimethod_form` + `parse_defmultimethod_form`.
6. **`src/freeze.rs:828-841`** — `is_mutation_form`. You add
   `:wat::core::defmultimethod` to this list so the freezer
   recognizes it as a top-level construct.
7. **`src/runtime.rs:685-725`** (SymbolTable struct) +
   **`src/runtime.rs:780-790`** (set_macro_registry pattern) —
   the capability-carrier pattern for the new
   `multimethod_registry` field.
8. **`src/runtime.rs:6267+`** — arc 144's `Binding` enum + the
   3 reflection primitive dispatch arms. You add a 6th `Binding`
   variant + extend each primitive's match.
9. **`src/runtime.rs:6315+`** — arc 144's `lookup_form`. You add
   a 6th branch consulting the multimethod registry.
10. **`src/check.rs:8323+`** — `unify` signature. You use this
    for arm-pattern matching at check-time.
11. **`src/special_forms.rs`** — arc 144 slice 2's special-form
    registry. You add `:wat::core::defmultimethod` registration
    so reflection sees it as a special form.

## What to ship

### 1. NEW module `src/multimethod.rs`

Mirror `src/macros.rs` shape:

```rust
//! Arc 146 slice 1 — multimethod entity + registry + parsing.
//!
//! A multimethod is a substrate entity that dispatches over
//! input type to one of N per-Type implementations. Pass-through
//! semantics: the multimethod's arity equals each arm's impl
//! arity; all args at the call site flow unchanged to the matched
//! impl. See arc 146 DESIGN.

use crate::ast::{Span, WatAST};
use crate::types::TypeExpr;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Multimethod {
    pub name: String,
    pub arms: Vec<MultimethodArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MultimethodArm {
    pub pattern: Vec<TypeExpr>,    // input-type pattern (one per arg)
    pub impl_name: String,          // keyword path of per-Type impl
    pub span: Span,
}

#[derive(Debug, Default, Clone)]
pub struct MultimethodRegistry {
    multimethods: HashMap<String, Multimethod>,
}

impl MultimethodRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn contains(&self, name: &str) -> bool { ... }
    pub fn get(&self, name: &str) -> Option<&Multimethod> { ... }
    pub fn register(&mut self, def: Multimethod) -> Result<(), MultimethodError> { ... }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Multimethod)> { ... }
}

pub enum MultimethodError {
    ReservedPrefix(String, Span),
    DuplicateMultimethod(String, Span),
    MalformedDefmultimethod { reason: String, span: Span },
    ArityMismatch { multimethod: String, surface_arity: usize, arm_impl: String, arm_arity: usize, span: Span },
}

pub fn is_defmultimethod_form(form: &WatAST) -> bool { ... }
pub fn parse_defmultimethod_form(form: WatAST) -> Result<Multimethod, MultimethodError> { ... }
```

The parsing form per the DESIGN:
```scheme
(:wat::core::defmultimethod :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
```

Each arm is `((<type-pattern>...) <impl-keyword>)` — arity of the
type-pattern is the multimethod's surface arity. The impl-keyword
points at a per-Type primitive that exists in CheckEnv (verify
existence at parse time? OR defer to first call? — see Q1 below).

### 2. Wire `MultimethodRegistry` into SymbolTable

Mirror the macro_registry pattern:

`src/runtime.rs` SymbolTable struct (line 685-725):
```rust
pub multimethod_registry: Option<Arc<crate::multimethod::MultimethodRegistry>>,
```

Setter (line ~785):
```rust
pub fn set_multimethod_registry(&mut self, registry: Arc<crate::multimethod::MultimethodRegistry>) {
    self.multimethod_registry = Some(registry);
}
```

Add `pub mod multimethod;` to `src/lib.rs`.

### 3. Freeze-time recognition

`src/freeze.rs:828-841` (`is_mutation_form`): add
`":wat::core::defmultimethod"` to the list.

In freeze.rs's main loop (wherever `:wat::core::defmacro` gets
processed — find via grep `is_defmacro_form` callers): add an
analogous arm for defmultimethod. Parse via
`crate::multimethod::parse_defmultimethod_form`; register into
the symbol table's MultimethodRegistry.

### 4. Check-time dispatch

`src/check.rs::infer_list` head-keyword switch (line 2956+): BEFORE
the first arm of the existing dispatch, check if the head is a
registered multimethod. If yes, route to multimethod arm-matching:

```rust
// Arc 146 slice 1 — multimethod dispatch.
if let Some(reg) = env.multimethod_registry() {
    if let Some(mm) = reg.get(k) {
        return infer_multimethod_call(mm, args, head_span, env, locals, fresh, subst, errors);
    }
}
```

The `env.multimethod_registry()` accessor needs to exist on
CheckEnv — add it (read SymbolTable's multimethod_registry through
the existing CheckEnv plumbing). Pattern matches how check.rs
accesses other SymbolTable data.

`infer_multimethod_call` body sketch:
```rust
fn infer_multimethod_call(
    mm: &Multimethod,
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // 1. Verify arity: args.len() must match mm.arms[0].pattern.len()
    //    (all arms have same arity by construction).
    // 2. Infer types for each arg.
    // 3. For each arm:
    //    - Try to unify each arg's inferred type with the arm's
    //      pattern (using existing `unify`).
    //    - If all unify, this arm matches.
    // 4. If no arm matches: clean TypeMismatch listing all arm patterns.
    // 5. If matched: instantiate the matched arm's impl scheme via
    //    the existing instantiate machinery; the call's return type
    //    is the impl's instantiated return type.
}
```

### 5. Runtime dispatch

`src/runtime.rs::eval_list_call` (or wherever the head-keyword
switch for runtime calls lives — search for the dispatch site at
line 2400+): BEFORE the existing arms, check the multimethod
registry similarly. If matched, dispatch to the matched arm's
impl.

```rust
// Arc 146 slice 1 — multimethod dispatch at runtime.
if let Some(reg) = &sym.multimethod_registry {
    if let Some(mm) = reg.get(k) {
        return eval_multimethod_call(mm, args, list_span, env, sym);
    }
}
```

`eval_multimethod_call` body sketch:
```rust
fn eval_multimethod_call(
    mm: &Multimethod,
    args: &[WatAST],
    span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // 1. Eval each arg.
    // 2. For each arm:
    //    - Match each arg's value tag against the arm's pattern.
    // 3. If matched: call the arm's impl with the same args (via
    //    the existing call-dispatch machinery).
    // 4. If no arm matches: RuntimeError::MultimethodNoMatch with
    //    the actual arg types for diagnosis.
}
```

### 6. Reflection (arc 144 extension)

`src/runtime.rs:6267` — `Binding` enum gains 6th variant:
```rust
Multimethod {
    name: String,
    mm: &'a crate::multimethod::Multimethod,
    doc_string: Option<String>,
},
```

`src/runtime.rs:6315+` — `lookup_form` gains 6th branch BEFORE
the SpecialForm branch (so multimethods take precedence over
special-form names if there's any collision — there shouldn't be):
```rust
if let Some(reg) = &sym.multimethod_registry {
    if let Some(mm) = reg.get(name) {
        return Some(Binding::Multimethod {
            name: name.to_string(),
            mm,
            doc_string: None,
        });
    }
}
```

The 3 reflection primitives (`eval_lookup_define`,
`eval_signature_of`, `eval_body_of`) gain match arms for the
Multimethod variant:
- `lookup_define`: emit the full `(:wat::core::defmultimethod :name (arms...))` form
- `signature_of`: emit the same (or a header-only variant)
- `body_of`: emit `:None` (multimethods have no "body" in the
  function sense; the dispatch table IS the contract)

Add a NEW helper `multimethod_to_define_ast(mm: &Multimethod) -> WatAST`
that builds the declaration form.

### 7. Special-form registration

`src/special_forms.rs` — add `:wat::core::defmultimethod` to the
registry with a sketch:
```
(:wat::core::defmultimethod <name> <arm>+)
;; <arm> ::= ((<type-pattern>...) <impl-keyword>)
```

So reflection sees defmultimethod as a special form (alongside
defmacro, define, struct, etc.).

### 8. Tests

NEW `tests/wat_arc146_multimethod_mechanism.rs` with 6+ tests:

The tests use a SYNTHETIC multimethod over leaf types so they
don't depend on any existing primitive being migrated. Example
shape:

```scheme
;; Two per-Type impls (clean rank-1 schemes — already work today)
(:wat::core::define
  (:test::i64-describe (x :wat::core::i64) -> :wat::core::String)
  "i64-arm")

(:wat::core::define
  (:test::f64-describe (x :wat::core::f64) -> :wat::core::String)
  "f64-arm")

;; Declare the multimethod
(:wat::core::defmultimethod :test::describe
  ((:wat::core::i64) :test::i64-describe)
  ((:wat::core::f64) :test::f64-describe))

;; Calls
(:test::describe 42)      ;; → "i64-arm"
(:test::describe 3.14)    ;; → "f64-arm"
```

Tests:
1. **`multimethod_dispatches_to_i64_arm`** — call with i64; assert "i64-arm" output.
2. **`multimethod_dispatches_to_f64_arm`** — call with f64; assert "f64-arm" output.
3. **`multimethod_no_arm_match_check_time`** — call with String; assert TypeMismatch at check.
4. **`lookup_form_returns_multimethod_binding`** — `(:wat::runtime::lookup-define :test::describe)` returns Some; AST contains `:wat::core::defmultimethod`.
5. **`signature_of_multimethod_returns_declaration`** — signature-of on the multimethod returns the declaration form.
6. **`body_of_multimethod_returns_none`** — body-of returns :None.
7. **(BONUS)** **`defmultimethod_arity_mismatch_errors`** — declare a multimethod where one arm's impl has different arity than the multimethod's surface arity; assert MultimethodError::ArityMismatch.

Tests follow `tests/wat_arc144_lookup_form.rs` shape.

### 9. Workspace verification

```
cargo test --release --test wat_arc146_multimethod_mechanism    # new tests pass
cargo test --release --test wat_arc144_lookup_form              # 9/9 unchanged
cargo test --release --test wat_arc144_special_forms            # 9/9 unchanged (or 10/10 if defmultimethod adds a row)
cargo test --release --test wat_arc144_hardcoded_primitives     # 17/17 unchanged
cargo test --release --test wat_arc143_lookup                   # 11/11 unchanged
cargo test --release --test wat_arc143_manipulation             # 8/8 unchanged
cargo test --release --test wat_arc143_define_alias             # 2/3 (length canary unchanged — slice 2 closes)
cargo test --release --workspace                                 # baseline failure profile + new tests
```

ZERO new regressions. The length canary stays red (slice 2 closes
it). The workspace failure profile is otherwise unchanged.

```
cargo clippy --release --all-targets
```

No new warnings.

## Constraints

- **NEW Rust file:** `src/multimethod.rs` (mirroring `src/macros.rs` shape).
- **EDITS:** `src/lib.rs` (`pub mod multimethod;`), `src/runtime.rs` (SymbolTable field + setter; Binding 6th variant; lookup_form 6th branch; reflection primitive arms; runtime dispatch insertion + helper), `src/check.rs` (CheckEnv multimethod accessor; infer_list dispatch insertion + `infer_multimethod_call` helper), `src/freeze.rs` (is_mutation_form + dispatch arm calling into multimethod parse + register), `src/special_forms.rs` (defmultimethod registration).
- **NEW test file:** `tests/wat_arc146_multimethod_mechanism.rs`.
- **NO migration of any existing primitive.** Slice 1 is mechanism only.
- **NO commits, no pushes.**

## Open questions sonnet must decide

### Q1 — Arity check at parse time vs first call

Should `parse_defmultimethod_form` look up each arm's impl in
CheckEnv to verify arity matches? Or defer until check-time
dispatch?

PARSE-TIME advantages: clear error at declaration site; no surprise
later. PARSE-TIME challenges: needs CheckEnv access at parse time
(may not be available; freeze typically runs before full env is
built).

**Recommendation:** check at FIRST CHECK-TIME CALL, not at parse
time. Surface as `MultimethodError::ArityMismatch` with both the
multimethod's declaration site span and the arm's impl span.
Defer parse-time enforcement unless the freeze ordering supports
it cleanly.

Sonnet decides + reports the choice.

### Q2 — Where do we put `infer_multimethod_call` + `eval_multimethod_call`?

In `check.rs` and `runtime.rs` respectively (alongside other
infer_* / eval_* helpers). Keeps the dispatch logic adjacent to
the dispatch sites. Sonnet places where the existing patterns
suggest.

### Q3 — Multimethod precedence vs special forms

If a name is BOTH a multimethod and a special form (shouldn't
happen but cover the case), which wins in lookup_form?

**Recommendation:** multimethod wins (it's user-declarable; special
forms are substrate-fixed). Add a comment naming the precedence.

### Q4 — Test file location for the test impls

The test multimethod's per-Type impls (`:test::i64-describe`,
`:test::f64-describe`) are user defines — they live in the test's
embedded wat string, not in the substrate. The multimethod
declaration is also in the embedded wat string. Standard test
shape.

Verify: defmultimethod can be declared after the impls are
defined, even if both are in the same wat source. Freezer must
process them in the right order (defmultimethod must see the
impls already registered, OR multimethod registration must be
deferred until after all defines).

If freezer order is a problem: surface as a Mode B-freeze-order
diagnostic; orchestrator decides scope.

## What success looks like

1. `src/multimethod.rs` exists with Multimethod + MultimethodArm + MultimethodRegistry + MultimethodError + parse functions.
2. SymbolTable carries `multimethod_registry` field + setter.
3. `:wat::core::defmultimethod` parses + registers at freeze time.
4. Check-time dispatch correctly routes multimethod calls to the matched arm's scheme.
5. Runtime dispatch correctly routes multimethod calls to the matched arm's impl.
6. Arc 144 reflection sees multimethods (Binding::Multimethod + lookup_form 6th branch + 3 reflection primitives handle the variant).
7. `:wat::core::defmultimethod` registered as a special form per arc 144 slice 2's registry.
8. New test file `tests/wat_arc146_multimethod_mechanism.rs` with 6+ tests; ALL pass.
9. ALL slice 1 + slice 2 + slice 3 + arc 143 baseline tests still pass.
10. `cargo test --release --workspace` failure profile unchanged (only the slice 6 length canary remains red).
11. `cargo clippy --release --all-targets` no new warnings.

## Reporting back

Target ~300-400 words.

1. **Multimethod struct + registry shape** — quote the verbatim
   declarations.
2. **The defmultimethod parse path** — name the file:line of
   `is_defmultimethod_form` + `parse_defmultimethod_form`.
3. **The check-time + runtime dispatch insertion** — quote the
   verbatim guard checks added to infer_list + eval_list_call.
4. **The arc 144 extensions** — quote Binding::Multimethod variant +
   the lookup_form 6th branch + each of the 3 reflection
   primitive's new arm.
5. **The defmultimethod special-form registration** — quote the
   sketch added to special_forms.rs.
6. **Test totals** — 6+ new tests pass; arc 144 + arc 143
   baselines unchanged; workspace failure profile unchanged.
7. **clippy** — quote any warnings (expected: none).
8. **Decisions on the open questions** — name what you chose for
   Q1 (parse-time vs first-call arity check), Q2 (helper
   placement), Q3 (precedence), Q4 (freeze ordering).
9. **Honest deltas** — anything you needed to investigate / adapt.

## Sequencing

1. Read pre-reads in order.
2. Create `src/multimethod.rs` mirroring `src/macros.rs` shape.
3. Wire MultimethodRegistry into SymbolTable.
4. Add freeze-time parsing (mirror defmacro flow in freeze.rs).
5. Add `:wat::core::defmultimethod` to special_forms.rs registry.
6. Add check-time dispatch (insertion in infer_list + helper).
7. Add runtime dispatch (insertion in eval_list_call + helper).
8. Add arc 144 extensions (Binding::Multimethod + lookup_form
   branch + 3 reflection primitive arms + helper).
9. Create `tests/wat_arc146_multimethod_mechanism.rs` with 6+
   tests.
10. Run `cargo test --release --test wat_arc146_multimethod_mechanism`
    — confirm new tests pass.
11. Run baseline tests (arc 144 + arc 143 suites) — confirm zero
    regression.
12. Run `cargo test --release --workspace` — confirm baseline
    failure profile.
13. Run `cargo clippy --release --all-targets` — confirm clean.
14. Report.

Then DO NOT commit. Working tree stays modified.

## Why this slice matters

This is the GATING substrate change for arc 146. After this slice,
the multimethod entity exists; future slices migrate one primitive
family at a time using the mechanism. The substrate gains an
honest representation for polymorphic dispatch — no more lying
schemes propped up by hidden handlers.

Per § 12 foundation discipline: this is the foundation auditing
itself. The slow path of doing it correctly compounds. Each
migration slice (slices 2-7) is short BECAUSE the mechanism does
the heavy lifting. Slice 1 is the investment.
