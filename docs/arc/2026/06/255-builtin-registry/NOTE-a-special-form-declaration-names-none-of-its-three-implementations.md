# NOTE (arc 255) — a special-form declaration names NONE of its three implementations, and `show-source` could

**Filed 2026-08-28, immediately after Stone P2, at the builder's question:** *"is there an `if` def
we should provide via a macro or something?.... the actual rust code that impls `if`?.... idk....
doesn't ruby's pry have a thing like this to show the c code?"* **A POINTER, not a decision.**

## Yes — and we already do it for 380 verbs

`pry` with the `pry-doc` gem shows the C source of a C-implemented Ruby method: `show-source
Array#push` prints `rb_ary_push`. **wat already has this for intrinsics** and it works today:

```
(:wat::core::show-source :wat::i64::+)
  =>  pub(crate) fn eval_i64_add(a: &WatAST, b: &WatAST, env: &Environment, …) -> … {
          const OP: &str = ":wat::i64::+";
          crate::runtime::eval_i64_arith(OP, &[a.clone(), b.clone()], span, env, sym, …)
      }
```

`#[wat_intrinsic]` captures the annotated fn with `quote!(#item).to_string()` into
`IntrinsicSubmission::source`, and `show-source` returns it. **The declaration IS the
implementation**, so the source comes along for free.

## Special forms are the gap — and the reason is structural, not an oversight

Stone P2 made `(:wat::core::show-source :wat::core::if)` stop returning `""` and start saying
*"no source available in this context"*. **That is honest but it is not true** — the source exists.
`#[wat_special_form]` annotates a **unit struct**:

```rust
#[wat_special_form(":wat::core::if")]
pub(crate) struct If;
```

There is no fn to capture, so `source: ""` is hardcoded in the fold. The declaration is a
**doc-only stub, divorced from the code that runs.**

## What `if` actually is — measured, and perfectly symmetric

| form | phase | fn | site |
|---|---|---|---|
| `:wat::core::if` | check | `infer_if` | `src/check.rs:7444` |
| | eval | `eval_if` | `src/runtime.rs:9221` |
| | tail (TCO) | `eval_if_tail` | `src/runtime.rs:4554` |
| `:wat::core::let` | check | `infer_let` | `src/check.rs:7718` |
| | eval | `eval_let` | `src/runtime.rs:8682` |
| | tail (TCO) | `eval_let_tail` | `src/runtime.rs:4611` |

**Three named Rust functions each, across two phases, and the declaration names none of them.**

★ **So wat's honest answer is RICHER than pry's, not poorer.** A Ruby C method is one C function and
`pry-doc` shows it. A wat special form is a *type rule* plus an *evaluation rule* plus a *tail
position rule* — and a reader asking "what is `if`" is better served by all three, labelled, than by
any one of them. `show-source` on a special form should print a three-part answer.

## The shape that would work — and it is this arc's own thesis

The macro cannot reach across files: a proc-macro sees only the tokens of the item it annotates, so
`#[wat_special_form(":wat::core::if", eval = eval_if)]` **cannot** capture `eval_if`'s body.

**Put the declaration ON each implementation**, exactly as `#[wat_intrinsic]` does:

```rust
#[wat_special_form_impl(":wat::core::if", role = check)]   fn infer_if(…)      { … }
#[wat_special_form_impl(":wat::core::if", role = eval)]    fn eval_if(…)       { … }
#[wat_special_form_impl(":wat::core::if", role = tail)]    fn eval_if_tail(…)  { … }
```

Each submits its own `quote!`-captured source through `inventory`, keyed by (FQDN, role) — the same
mechanism already carrying 380 intrinsic sources. The registry gathers the three; `show-source`
prints them labelled. **The declaration stops being a stub and becomes what it describes.**

⚠ This changes `IntrinsicEntry::source: &'static str` into something plural. Do not assume the shape;
that is the design decision, and it interacts with `SpecialFormSubmission` and with how `render-doc`
reads the entry.

## ★ The larger prize, and the reason this may be worth more than a reflection nicety

**Only 2 special forms are registered at all** (`if`, `let` — anchored count). Every other special
form in the language is a bare match arm in `runtime.rs`/`check.rs` that the registry has never heard
of. An annotation that lives on the *implementation* is one a form cannot be written without — so
this is not only a `show-source` fix, it is a route to making the remaining special forms
**addressable**, which is what `walk.rs:268` needs and what arc 255 exists for.

⚠ **The size of "every other special form" is NOT measured here.** The 294 seam carries a figure of
36 among the remaining dispatch arms; this note did not validate it and does not repeat it as fact.
Counting them with a validated instrument is step one of any stone that follows.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## What this note does NOT decide

Whether to build it, what the captured shape is, whether `role` is the right axis (a form with no
TCO arm has two, not three), and whether it precedes or follows the O-iv sweep. All the builder's.

## Refs

- `crates/wat-macros/src/wat_intrinsic.rs` — `quote!(#item).to_string()` into `source`; the
  mechanism to mirror. `crates/wat-macros/src/wat_special_form.rs` — what would change.
- `src/intrinsic/special/{control_flow,binding}.rs` — the two unit-struct declarations.
- `src/intrinsic/reflect.rs` — `show-source`, and P2's `Kind::SpecialForm` gate that would give way
  to a real answer.
- `NOTE-an-absence-recorded-as-an-answer-…md` — the class; `source: ""` was finding 2.
