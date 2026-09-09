//! Special-form doc entry for `:wat::eval-ast!` — arc 255 Stone ③a-i, the first of ③a's three
//! eval-verb mechanism groups and the TEMPLATE the other two (`③a-ii`, `③a-iii`) copy. `eval-ast!`
//! is the FENCED member (`run_constrained`): the only eval verb whose mechanism differs from
//! every sibling, so its axes are argued fresh here rather than inherited.
//!
//! ⛔ THE AXES THIS FILE ALMOST DECLARED, AND WHY THEY WERE WRONG — write it down so the next
//! self does not re-litigate it. Measured, on the green tree, by RUNNING:
//! `(:wat::eval-ast! '(:wat::kernel::println "B"))` prints `"B"`; `(:wat::eval-ast! '(:wat::i64::/
//! 1 0))` returns `Err` and execution CONTINUES; `(:wat::eval-ast! '(Option/expect
//! (Option.None{}) …))` lets the panic ESCAPE (exit 2, "after" never runs); `(:wat::eval-ast!
//! '(:wat::core::typealias …))` returns `Err "eval refused mutation form"`. From the first and
//! third, `@Purity Effectful` / `@Determinism Nondeterministic` / `@Totality Partial` looked
//! right — reasoning that `Preserving`'s prose inherits from *sub-forms*, and an evaluand is a
//! runtime VALUE, not a sub-form.
//!
//! `:wat::core::apply` (`src/runtime.rs:5260`) already settled this the other way: its dispatch
//! target is ALSO a runtime value the head merely names, and it declares `Preserving` on all
//! three anyway — `Preserving` asks "does this verb add an effect of its own?", not "can the
//! checker see the subject?" The effect measured above is the EVALUAND's, exactly as `apply`'s
//! target's would be. Four of the five axes below are `apply`'s, for `apply`'s reasons — the
//! same mechanism (control transferred into a runtime-supplied subject) — so a different verdict
//! would be the thing needing an argument.

use wat_macros::wat_special_form;

/// Evaluate the already-parsed AST `<form>` (a `:wat::WatAST` value, typically produced by
/// `:wat::core::quote` or `:wat::edn::read`) against the caller's ambient environment, first
/// refusing any declaration/mutation head found anywhere in the form's tree
/// (`run_constrained` → `refuse_mutation_forms_in`: `defmacro`, `defstruct`, `structtype`,
/// `defenum`, `newtype`, `typealias`, `load-file!`, `digest-load!`, `signed-load!`, and any
/// `:wat::config::set-*` head) — a mutation-headed evaluand comes back as `Err`, not a raise.
/// Past that fence, the evaluand runs by ordinary evaluation; a successful result is wrapped
/// `Ok`, and a caught runtime error is wrapped `Err` as a `:wat::core::EvalError` — the same
/// `Result<T, EvalError>` shape an `eval-edn!`/`eval-file!` caller already expects.
///
/// **Category ground —** `:ControlFlow`'s own prose — "directs evaluation (`if`, and applying a
/// callable handed in as a value)". `eval-ast!` directs evaluation into a form handed in as a
/// value; `apply` is named verbatim by that prose for the identical act with a callable instead
/// of a form. `ControlFlow`.
///
/// **Purity ground — `Preserving`, not `Effectful`:** measured (see the module doc's
/// retraction, above) that the one observable effect a run can produce —
/// `(:wat::kernel::println "B")` printing `"B"` — is the EVALUAND's effect, not `eval-ast!`'s
/// own; `eval-ast!`'s own body performs no I/O, mutation, or entropy read past dispatching
/// whatever `run_constrained` is handed. Same ground `apply` (`src/runtime.rs:5260`) argues for
/// its own runtime-resolved dispatch target — a form whose purity is its sub-forms', not its
/// own, extended here from a callable to an arbitrary evaluand. `Preserving`.
///
/// **Determinism ground — `Preserving`, same ground:** given a deterministic evaluand
/// `eval-ast!` is deterministic; given `(:wat::time::now)` as the evaluand it is not — it
/// contributes neither determinism nor nondeterminism of its own, only whatever the evaluand
/// carries. `Preserving`.
///
/// **Totality ground — `Preserving`:** measured that a partial evaluand's panic passes THROUGH
/// (`Option/expect` on `None`: exit 2, "after" never runs) — inherited, not added. ⚠ The
/// mutation fence does NOT make this `Partial`: `refuse_mutation_forms_in` returns an `Err`
/// VALUE for a mutation-headed evaluand, which is a total answer (a matchable outcome), not a
/// domain hole — the same arity/shape-guard carve-out `apply`'s own `Preserving` ground uses,
/// and the same raise-vs-constructed-`Err` distinction
/// `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md` draws. `Preserving`.
///
/// **Expand-time ground — `RuntimeOnly`:** `:RuntimeOnly`'s own definition
/// (`wat/runtime-meta.wat`) names this case VERBATIM — "needs state that does not exist yet at
/// expand time — … or THE EVALUATION OF ARBITRARY SUBMITTED FORMS." Corroborated independently:
/// every `@Purity` `Effectful`-or-`Preserving` verb behind a runtime indirection is denied by the
/// expand-time walker, and `apply` is named in that residue header by name — `eval-ast!` is the
/// same indirection one level more literal (the walker is handed a runtime VALUE it cannot see
/// through, not even a keyword leaf naming the eventual verb, the way `mapv`'s literal-AST `f`
/// argument can be). `RuntimeOnly`.
///
/// @added         1.0.0
/// @Purity        Preserving
/// @Determinism   Preserving
/// @Totality      Preserving
/// @ExpandTime    RuntimeOnly
/// @Category      ControlFlow
/// @syntax  (:wat::eval-ast! <form>)
/// @ret     (:wat::core::Result :- [T :wat::core::EvalError]) the evaluand's own value in `Ok`, or a caught runtime error as `Err`
/// @example (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1 2))) #=> (:wat::core::Result::Ok {:value 3})
#[wat_special_form(":wat::eval-ast!")]
pub(crate) struct EvalAst;
