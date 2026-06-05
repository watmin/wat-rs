# BRIEF — Stone 249.3a-ii — form decomposition: `first`/`rest` over a form-value

**Arc:** 249 (total-pure-macros). **Design:** `DESIGN-STONE-249.3.md` §2 (form vocabulary) + the row-F finding (commit `342374a4`).
**Probe contract:** `tests/probe_arc249_threading_in_wat.rs` row F (`diag_thread_first`) + rows A/B/C/E (must stay green).
**You write substrate Rust. Do NOT commit. Do NOT run git. Do NOT touch any wat/ file. Leave the collection-home re-ward to the orchestrator.**

---

## The goal — make `first`/`rest` decompose a form-value

Thread-first (`(-> 5 (i64::- 3))` → `(i64::- 5 3)`) injects the accumulator as the FIRST arg of a step form, which requires the step DECOMPOSED: head via `first`, tail via `rest`, rebuilt `(head acc tail…)`. In the macro program-body context, a step binds as `Value::wat__WatAST(List)` (proven — the `~@step` splice fires the `wat__WatAST(List)` arm). But `first`/`rest` reject it today:

```
:wat::core::first: expected tuple, Vec, or List, got wat::WatAST
```

`first`/`rest` decompose `Vec` and `wat__core__List` but NOT `wat__WatAST` form-values — an asymmetry (`feedback_asymmetries_meet_high_bar`). A macro program over forms is the seq abstraction's home case (Clojure: `(first '(f a b))` → `f`). Complete the family.

## Two changes — add a `wat__WatAST(List)` arm to each

### 1. `eval_positional_accessor` (`src/runtime.rs:12311`) — covers `first`/`second`/`third`

It currently matches `Tuple`, `Vec`, `wat__core__List`, else `TypeMismatch`. Add an arm:

```rust
Value::wat__WatAST(ast) => match &*ast {
    WatAST::List(children, _) => Ok(Value::Option(Arc::new(
        children.get(index).cloned().map(|c| Value::wat__WatAST(Arc::new(c))),
    ))),
    // A non-List form has no positional children — None (matches Vec out-of-bounds shape).
    _ => Ok(Value::Option(Arc::new(None))),
},
```

Returns `Option<wat__WatAST>` (projective, matching the Vec/List arms' `Option<T>` shape). The head/Nth child is returned AS a form-value.

### 2. `eval_vec_rest` (`src/collection/eval.rs:874`) — `rest`

It currently matches `Vec` and `wat__core__List` (errors on empty), else `TypeMismatch`. Add a `wat__WatAST(List)` arm that returns the **tail as a form** (maintains form identity, mirroring the 220.4 "List/rest → List" precedent at lines 899-910):

```rust
Value::wat__WatAST(ast) => match &*ast {
    WatAST::List(children, span) => {
        if children.is_empty() {
            return Err(/* RuntimeErrorKind::MalformedForm, "cannot take rest of empty form",
                         mirror the empty-List arm */);
        }
        let tail: Vec<WatAST> = children.iter().skip(1).cloned().collect();
        Ok(Value::wat__WatAST(Arc::new(WatAST::List(tail, span.clone()))))
    }
    other_ast => Err(/* TypeMismatch: rest of a non-List form, "expected a list form" */),
},
```

`rest` of `(f a b)` → the form `(a b)`; `rest` of `(f)` → the form `()` (empty tail, NOT an error — `(f)` has one element). `~@(rest step)` then splices the tail's children (the splice handles `wat__WatAST(List)`).

## Verification (the scorecard — run every row yourself, report actual output)

1. **Thread-first works** — un-`#[ignore]` row F (`diag_thread_first`) in `tests/probe_arc249_threading_in_wat.rs`; `cargo test --release --test probe_arc249_threading_in_wat` → row F green (`(-> 5 (i64::- 3))` → 2). Rows A/B/C/E stay green; row D stays `#[ignore]`'d.
2. **Engine contract** — `cargo test --release --test probe_arc249_macro_engine` gates A–E green (no regression).
3. **Collection home tests intact** — `cargo test --release` for any `first`/`rest`/`second`/`third` collection tests (grep `tests/` for them) — the new arm must not change Vec/List behavior.
4. **Library** — `cargo test --release --lib -p wat` → ≥ 898/0/1 (no drop).
5. **Clippy** — `cargo clippy --release -p wat` → zero new warnings on your touched lines.

Report each row's command + output. If a pre-existing test goes red, STOP and report it as a finding.

## Notes
- Bash + cargo work; use them freely.
- Two located arms only: `src/runtime.rs` (`eval_positional_accessor`) + `src/collection/eval.rs` (`eval_vec_rest`). No new files, no new error variants (reuse `TypeMismatch` / `MalformedForm`), no `wat/` edits.
- `src/collection/` is a WARDED home (arc 246) — your arm drifts its stamp; the orchestrator re-wards it after verifying your work. Just make the arm correct, obvious, and mirror the existing `wat__core__List` arm's shape.