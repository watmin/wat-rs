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


## ⛔ THE BUILDER'S FOLLOW-UP — is there a higher-order entrypoint that calls the three?

> *"is this hinting that we need a higher order intrinsic entrypoint who calls these three?..
> otherwise..... the (check, eval, tail) /feels/ ok... but idk.."*

**No — and the reason is precise. The three do not compose, so there is nothing for an entrypoint to
call.** They are not three steps of one operation; they are the same form's rule under three
regimes:

```
check   runs ONCE, statically, before any evaluation exists      (src/check.rs)
eval    runs per-invocation                                       ┐ MUTUALLY EXCLUSIVE —
tail    runs per-invocation, in tail position only                ┘ selected by POSITION, never both
```

A fn calling all three would have to run at two different times and pick between two of them. There
is no such call. **The triple is right; the builder's instinct that it "feels ok" is correct.**

## ★ WHAT THE INSTINCT IS ACTUALLY POINTING AT — one record, not one call

What is wrong is not the decomposition, it is that **the triple lives in three hand-maintained
places** and a form must be added to each independently:

```
src/check.rs      the infer_* match          ← rule 1
src/runtime.rs    the eval match             111 arms
src/runtime.rs    the tail match             8 arms — if · match · let · do · and · or · ann-form · rete::insert
```

The unifying shape is not a function. It is **a record — which is what a registry entry already
is**:

```
special form = { check: Rule, eval: Rule, tail: Option<Rule> }
```

Two parallel hand-written matches collapse into one lookup, and the (FQDN, role) submissions
proposed above are exactly the rows of that record.

★ **AND THE PRECEDENT IS ALREADY ON DISK, IN THE TAIL MATCH'S OWN COMMENT.**
`:wat::kernel::serve-dispatch-op` used to be a hardcoded arm in the tail match. Arc 255's kernel
home moved it into the intrinsic registry, and the tail match's fallthrough —
`_ => eval_inner(ast, env, sym)` — now reaches it by registry lookup, calling the same tail delegate
and preserving the `serve` trampoline. **One special form already lives the way this note proposes.**
The question is not whether the shape works; it is whether to finish it.

## ⚠ AND `tail: Option<Rule>` IS AN HONEST `None` — the contrast is the class's discriminator

This whole NOTE family is about absences recorded as answers, so the `Option` deserves the test —
and it **passes**, which is worth writing down because it shows where the line is:

| | what `None` means | what it produces |
|---|---|---|
| `value_handler: None` | *nobody wrote one* | **a LIE** — `apply` reported 331 registered verbs absent |
| `tail: None` | *no tail rule* | falls through to `eval_inner`: **correct, just not tail-optimized** |

**The discriminator: does the absence produce a WRONG answer, or a slower-but-right one?** A missing
tail rule costs stack depth on deep tail recursion; it never gives the wrong value. That is a real,
safe default — the kind an `Option` is allowed to carry.

⚠ **The residual question this note does NOT answer:** nothing records whether a form *needs* a tail
rule. 8 of the forms have one; a ninth that recurses in tail position without one will consume the
Rust stack at depth. There IS a TCO gate (`tests/rete/probe_arc278_55_slice_one_vocabulary.rs`, cited
in the tail match's comment, with a measured 200,000-deep case) — whether it covers the population or
only its own subject is **unmeasured here**, and is the same question P4 asks of the checker gates.
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

---

## ⛔ SHIPPED 2026-08-28 (P6-a) — and it turned two buried lies into published ones the same hour

The mechanism landed on `if` and `let`. **The first thing it did was publish an inverted doc comment
to users**, which is a consequence worth carrying forward as a rule:

`eval_if`'s doc read *"Arity: exactly 5 args. Positions: [cond, `->`, `:T`, then, else]. The old
3-arg form is refused"* — **the precise opposite of the code beneath it.** Arc 258.4 retired the
`-> :T` ascription; `args.len() == 3` is the live path and a stray `->` is what gets refused.
`infer_if` carried the same inversion. Both were corrected in the same commit.

> ★ **A fn named by `#[wat_special_form_impl]` has USER-FACING DOCUMENTATION in its doc comment.**
> `show-source` prints the captured item, `#[doc = …]` attributes and all. Before P6-a that comment
> was internal prose that only a Rust reader met; after it, it is what a wat programmer reads when
> they ask what `if` is. **Stale prose on these fns is a shipped lie**, in exactly the sense the
> `circumspicere` cast ranked highest — and the same is already true of all 380 `#[wat_intrinsic]`
> handlers, which have been publishing their doc comments this whole time.

The two inversions had been buried since arc 258.4. Nothing found them; making the source reachable
did, within an hour. `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

## ⚠ AND A CENSUS BLIND SPOT FOR P6-c, found by the rider

`:wat::rete::insert` appears in the 8-arm tail match but is dispatched in the eval path by a
**pre-match `if head == … { return … }` short-circuit** (`src/runtime.rs:~5340`), not by a literal
match arm. **A line-anchored grep over the match body cannot see it.** P6-c's census must read the
function preamble, not only the arms — the same instrument-blindness class this arc keeps paying
for, one level removed: not prose mistaken for code, but code sitting *ahead of* the structure the
instrument was pointed at.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

⚠ The rider also measured the eval and check matches at **148 unique FQDN arms each**, intersecting
at 68 — and correctly refused to call that 68 "special forms": most are ordinary polymorphic
builtins (`get`, `assoc`, `conj`, `map`, `filter`, `HashMap`, …) that evaluate their arguments
eagerly and are merely hardcoded rather than macro-declared. The true syntactic subset — heads that
control *whether* their own subexpressions evaluate — is roughly a dozen, and **that dozen is not
validated either.** Sizing it properly is P6-c's row 0.
