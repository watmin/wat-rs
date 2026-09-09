# DESIGN — STONE ③a-i: `:wat::eval-ast!` declares its axes

The first of ③a's three mechanism groups, and the TEMPLATE the other two copy. One name, one
`#[wat_special_form]` declaration, five axes — each argued from a measurement or a precedent.

## Why this one first

`③a-i` is the FENCED member (`run_constrained`). It is the only eval verb whose mechanism differs
from every sibling, so its axes cannot be inherited from anything — which makes it the honest place
to establish the template. `[[DESIGN-STONE-3…]]` § "③a IS A CAMPAIGN".

## ⛔ THE AXES I WAS ABOUT TO DECLARE, AND WHY THEY WERE WRONG

Measured, on the green tree, by RUNNING:

```
(:wat::eval-ast! '(:wat::kernel::println "B"))       → prints "B"      → I read: EFFECTFUL
(:wat::eval-ast! '(:wat::i64::/ 1 0))                → Err division-by-zero, execution CONTINUES
(:wat::eval-ast! '(Option/expect (Option.None{}) …)) → the PANIC ESCAPES; exit 2, "after" never runs
(:wat::eval-ast! '(:wat::core::typealias …))         → Err "eval refused mutation form"
```

From the first and third I was going to declare **`@Purity Effectful`**, **`@Determinism
Nondeterministic`**, **`@Totality Partial`** — reasoning that `:Preserving`'s prose says it inherits
from *"sub-forms"*, and an evaluand is a runtime VALUE, not a sub-form.

★★★ **`:wat::core::apply` (`src/runtime.rs:5300`) already settled this, and it settles it the other
way.** Its dispatch target is *also* a runtime value the head merely names, and its recorded ground
is explicit:

> *"`apply`'s dispatch target is a RUNTIME VALUE the head expression merely NAMES … the walker
> cannot see through `apply`'s indirection to the verb it will actually invoke."* — and it declares
> `Preserving` on all three anyway, with **`@ExpandTime RuntimeOnly`** carrying the visibility
> problem as a **deliberately independent axis** (*"`apply`'s OWN effect is conditional on its
> target, but whether the WALKER can safely admit it during expansion is a static-visibility
> question this indirection fails regardless"*).

So `Preserving` asks **"does this verb add an effect of its own?"** — not *"can the checker see the
subject?"* The effect I measured was the **evaluand's**, precisely as apply's target's would be.
My three axes were wrong, and the precedent's own argument is what caught it.

## The ruled axes

```
@Category      ControlFlow    :ControlFlow's prose — "directs evaluation (if, and applying a
                              callable handed in as a value)". eval-ast! directs evaluation into
                              a form handed in as a value. apply is named verbatim by that prose;
                              this is the same act with a form instead of a callable.
@Purity        Preserving     Adds no effect of its own. Measured: the printed effect was the
                              evaluand's. Same ground as apply.
@Determinism   Preserving     Same ground. Given a deterministic evaluand it is deterministic;
                              given `(:wat::time::now)` it is not. It contributes neither.
@Totality      Preserving     Measured: a partial evaluand's panic passes THROUGH (exit 2) —
                              inherited, not added. ⚠ And the mutation fence does NOT make it
                              Partial: `refuse_mutation_forms_in` returns an `Err` VALUE, which
                              is a total answer, not a domain hole. Same carve-out apply names
                              for its arity/shape guards.
@ExpandTime    RuntimeOnly    `:RuntimeOnly`'s own definition (`wat/runtime-meta.wat`) names this
                              case VERBATIM: "needs state that does not exist yet at expand time
                              — … or THE EVALUATION OF ARBITRARY SUBMITTED FORMS."
                              Corroborated independently: every @Purity-Effectful-or-Preserving
                              verb behind a runtime indirection is denied by the expand-time
                              walker, and `apply` is named in that residue header by name.
```

★ Four of the five are `apply`'s, for `apply`'s reasons. That is not laziness — it is the same
mechanism (control transferred into a runtime-supplied subject), so a DIFFERENT verdict would be
the thing needing an argument.

## ⛔ CORRECTED AFTER THE FIRST STRIKE — TWO CLAIMS BELOW WERE FALSE

The declaration was built exactly as drawn and the floor refused it, **4 red at 5301/5305**. Every
refusal is one of this arc's own gates, working:

```
every_special_form_carries_check_and_eval_impls
    ":wat::eval-ast! — missing role: check / missing role: eval"
probe_can_doc_types_reconstruct_the_checker_scheme
    "a registered row's @arg/@ret types no longer reconstruct its checker TypeScheme"
registry_membership_gap_a / _b       the ratchets, now STALE for this name — mechanical
```

**① "No `NativeHandler`… the arm is untouched" — FALSE.** `#[wat_special_form]` is not a bare
declaration in this arc. A registered special form MUST carry `#[wat_special_form_impl]` pointers
for roles `check` and `eval`, which means its dispatch runs THROUGH the registry and its literal
arm in `runtime.rs:2956`'s ten-way match **dies**. That is
`[[BRIEF-STONE-the-eval-door]]`'s shape, in its own words: *"Give `role = eval` a callable pointer,
so the four registered special forms dispatch through the registry and their four literal arms
die."*

★ **That brief was open in front of me earlier the same session.** I read its opening paragraph
and moved on. `[[feedback_read_the_epitaph_before_you_build_on_prior_art]]`

**② "The contract is NOT restated" — FALSE.** `probe_can_doc_types_reconstruct_the_checker_scheme`
requires a registered row's `@arg`/`@ret` to RECONSTRUCT its checker `TypeScheme`. `check.rs:5823`'s
STOP-6 governs which authority WINS at check time; it does not excuse the doc from expressing the
same contract. The row and the scheme must agree, or the divergence goes on
`FROZEN_SPELLING_MISMATCHES` with a measured reason.

★ **The five axes are UNAFFECTED and stand** — they were argued from mechanism and precedent, and
no gate touched them. The retracted-axes note and the axis grounds in the first strike's doc block
(commit `62448d367`, reverted but kept) are the redraw's starting point.

## The mechanism — CORRECTED

`#[wat_special_form(":wat::eval-ast!")]` on a unit struct, in a new
`src/intrinsic/special/eval_ast.rs`, registered in `special/mod.rs`. Per-site declaration collected
by `inventory` — never a hand-list in the registry builder
(`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).

⛔ **PLUS the two role pointers the wall requires** — `#[wat_special_form_impl]` for `role = check`
and `role = eval` — and the retirement of `:wat::eval-ast!`'s literal arm from `runtime.rs:2956`'s
ten-way match. ⚠ That match peels the type binder ONCE for all ten members
(`let (_binder, args) = peel_param_spec(args)`); removing one member must not strand the other
nine, and the arm's own comment says the shared peel is deliberate. **Measure that before drawing
the strike.**

## ⛔ The contract is NOT restated

`:wat::eval-ast!` already carries a `TypeScheme` (`src/check.rs:19493`). Per `src/check.rs:5823` —
*"the scheme is strictly stronger and stays the one authority for that row (STOP-6: one authority
per question)"* — the row contributes MEMBERSHIP and DOCUMENTATION. `@ret` is prose-and-type for
`render-doc`; it does not become a second checking authority, and `Kind::SpecialForm` is excluded
from the row-arity path by that same passage.

## Out of scope = REJECTED

- **③a-ii and ③a-iii.** Different mechanisms, different arguments. This stone is the template.
- **The fence asymmetry** (`eval-with-defs!` performs declaration forms `eval-ast!` refuses).
  Recorded in the parent DESIGN as an open substrate question; `eval-ast!`'s own axes do not
  depend on it.
- **Retiring the hand-written scheme.** It is the stronger authority; the row does not replace it.
