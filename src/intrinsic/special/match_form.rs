//! Special-form doc entry for `:wat::core::match` — arc 255.SF, the-membership-gap-gets-a-ratchet.

use wat_macros::wat_special_form;

/// Evaluate `<scrutinee>`, then evaluate and return the body of the first arm whose pattern
/// matches it; each arm's pattern introduces bindings visible only in its own body. Untaken
/// arms are never evaluated — the same reason `if`'s untaken branch is not, and the reason
/// `match` is a special form rather than an ordinary function.
///
/// **Purity ground —** `match` has no purity of its own; it PRESERVES: the only two things
/// `match` itself runs are the scrutinee and the ONE taken arm, so it is pure exactly when both
/// are — the same sentence `Purity::Preserving` was minted with for `if` (`control_flow.rs`).
/// `Preserving`.
///
/// **Determinism ground —** identical reasoning: `match` runs exactly the scrutinee and the
/// chosen arm and nothing else, so it is deterministic exactly when both are. `Preserving`.
///
/// **Totality ground —** exhaustiveness is enforced at CHECK time, per the detected shape
/// (`Option`/`Result`/user enum/open hash-destructure — `infer_match`'s own doc, `check.rs`), so
/// a well-typed `match` always selects an arm at runtime; it is then total exactly when the
/// scrutinee and the selected arm are — the same `Preserving` shape `if` uses. `Preserving`.
///
/// **Expand-time ground —** `match` evaluates real sub-forms at its own call site (the scrutinee,
/// then one arm), so — unlike `fn`, which evaluates nothing and is unconditionally `Legal` — its
/// own expand-time legality genuinely depends on theirs: legal at expand time exactly when the
/// scrutinee and the taken arm are, mirroring `if`'s own `Preserving` ruling
/// (`macros/eval.rs`'s expand-time allow-list residue currently hand-lists `match` as
/// unconditionally legal for lack of a registration site to carry a real ruling; this stone
/// gives it one, from the same registry door, with the same net verdict — see the report's note
/// on that residue list). `Preserving`.
///
/// @added 1.0.0
/// @Category ControlFlow
/// @Purity Preserving
/// @Determinism Preserving
/// @Totality Preserving
/// @ExpandTime Preserving
/// @syntax (match <scrutinee> (<pattern> <body>) ...)
/// @ret :T the taken arm's value; every arm unifies to T
/// @example (:wat::core::match (:wat::core::Some 3) ((:wat::core::Some x) x) (:wat::core::None 0)) #=> 3
#[wat_special_form(":wat::core::match")]
pub(crate) struct Match;
