//! Special-form doc entry for `:wat::holon::literal` — arc 255 Stone
//! holon-literal-is-a-special-form. Reclassified from `#[wat_intrinsic]` (`Kind::Intrinsic`) to
//! `#[wat_special_form]` (`Kind::SpecialForm`): its own check arm captures `form` as DATA
//! without evaluating it — *exactly* `:wat::core::quote`'s shape (`quote.rs`, the sibling this
//! stone brings it in line with; both share `eval_quote` in `runtime.rs`, deliberately). The
//! `role = eval` pointer stays on `eval_holon_literal` in `intrinsic/holon/atom.rs` — that fn
//! already carries the canonical `NativeHandler` shape, so no wrapper is needed here, unlike
//! `quote`/`forms`.

use wat_macros::{wat_special_form, wat_special_form_impl};

use crate::ast::WatAST;
use crate::check::{CheckEnv, CheckError, CheckErrorKind, CheckResult, InferCtx, Subst};
use crate::span::Span;
use crate::types::TypeExpr;
use std::collections::HashMap;

/// `(:wat::holon::literal form)` -> `(:wat::holon::to-holon (:wat::core::quote
/// form))`, fused: quotes `form` without evaluating it, then lowers the
/// quoted form to a HolonAST composition directly (shares `:wat::core::quote`'s
/// `eval_quote`, which is genuinely shared between the two verbs and stays
/// in `runtime.rs`). Also spelled as the `#holon <form>` reader tag (arc 294.b) — the two
/// surface forms are the same verb.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @syntax  (:wat::holon::literal <form>)
/// @ret     :wat::holon::HolonAST the HolonAST composition encoding the form's structure
/// @example (:wat::holon::literal (f x)) #=> (:wat::holon::literal (f x))
/// @see     :wat::holon::from-wat
#[wat_special_form(":wat::holon::literal")]
pub(crate) struct HolonLiteral;

/// Arc 255 Stone holon-literal-is-a-special-form — the `role = check` pointer for
/// `:wat::holon::literal`. The inline arm at `check.rs` (formerly `":wat::holon::literal" =>
/// { ... }`) stays untouched in its OWN LOGIC (STOP-3: extracted verbatim), moved wholesale to
/// this named fn so the registry's `role = check` annotation names real, reachable code rather
/// than a fn nothing calls — the same wiring `:wat::core::quote`/`:wat::core::forms` got in
/// Stone 1a-gamma-i.
///
/// The enclosed form is DATA captured without evaluation (exactly as `:wat::core::quote`). The
/// checker does NOT recurse into the body — this is the entire point: heterogeneous EDN
/// maps/sets bypass monomorphic `infer_map_literal` because the type is declared as
/// `:wat::holon::HolonAST` at the head alone.
#[wat_special_form_impl(":wat::holon::literal", role = check)]
pub(crate) fn infer_holon_literal(
    _head: &str,
    args: &[WatAST],
    head_span: &Span,
    _env: &CheckEnv,
    _locals: &HashMap<String, TypeExpr>,
    _fresh: &mut InferCtx,
    _subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    let mut local_errors: Vec<CheckError> = Vec::new();
    if args.len() != 1 {
        local_errors.push(CheckError { span: head_span.clone(), kind: CheckErrorKind::ArityMismatch {
            callee: ":wat::holon::literal".into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ty = TypeExpr::Path(":wat::holon::HolonAST".into());
    if local_errors.is_empty() { CheckResult::ok(ty) } else { CheckResult::partial_with(ty, local_errors) }
}
