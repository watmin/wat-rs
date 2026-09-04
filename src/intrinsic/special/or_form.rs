//! Special-form doc entry for `:wat::core::or` — arc 255 Stone 1a-i,
//! the-registry-becomes-the-sole-authority.

use wat_macros::wat_special_form;

/// Evaluate each `<expr>` in order; return `:true` at the first one that is `:true` without
/// evaluating any that follow, or `:false` if every operand was `:false` (including the empty
/// case — `(or)` #=> false). The untaken trailing operands are never evaluated — the same reason
/// `if`/`match`'s untaken branches are not, and the reason `or` is a special form rather than an
/// ordinary function: an ordinary function's arguments are all evaluated before the call ever
/// happens. Sibling of `and` (`and_form.rs`) — same shape, short-circuiting on the opposite value.
///
/// **Purity ground —** `or` adds no effect of its own; only the operands it actually runs (every
/// operand up to and including the first `:true`, or all of them if none is) can have effects, so
/// it is pure exactly when those are — the same sentence `Purity::Preserving` was minted with for
/// `if` (`control_flow.rs`). `Preserving`.
///
/// **Determinism ground —** identical reasoning: the same operands, evaluated in the same order
/// up to the same short-circuit point (or to the end), always produce the same result; `or`
/// introduces no independent source of variation. `Preserving`.
///
/// **Totality ground —** `or` is total exactly when every operand it actually evaluates is total;
/// an operand skipped by the short-circuit contributes nothing, the same sentence `if`'s own
/// `Totality::Preserving` was minted with for its untaken branch. `Preserving`.
///
/// **Expand-time ground —** `or` evaluates real sub-forms at its own call site (every operand up
/// to the short-circuit point), so — unlike `fn`, which evaluates nothing at its own call site —
/// its own expand-time legality genuinely depends on theirs: legal at expand time exactly when
/// the operands it runs are, the same `Preserving` shape `match` uses for the identical reason
/// (`match_form.rs`). `macros/eval.rs`'s expand-time allow-list residue currently hand-lists `or`
/// as unconditionally legal for lack of a registration site to carry a real ruling; this stone
/// gives it one, from the same registry door, with the same net verdict — `is_expand_time_legal`
/// treats `Legal` and `Preserving` identically at its per-head check (`matches!(e.expand_time,
/// Legal | ExpandOnly | Preserving)`), so declaring `Preserving` here changes nothing observable
/// while being the honest label for a form that does evaluate its own sub-forms. `Preserving`.
///
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Totality Preserving
/// @ExpandTime Preserving
/// @syntax (:wat::core::or <exprs>+)
/// @arg exprs… :wat::core::bool the operands, evaluated left to right until the first `:true` (or all of them)
/// @ret :wat::core::bool `:true` at the first `:true` operand, else `:false` (`:false` when there are no operands)
/// @example (:wat::core::or false true) #=> true
/// @example (:wat::core::or false false) #=> false
/// @example (:wat::core::or) #=> false
#[wat_special_form(":wat::core::or")]
pub(crate) struct Or;
