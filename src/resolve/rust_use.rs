//! Collection of `:wat::core::use!` declarations and rust-deps coverage checks.
//!
//! Pass 1 of [`super::resolve_references`]: scan top-level forms for
//! `(:wat::core::use! :rust::...)` and validate each against the build-time
//! rust-deps registry.

use crate::ast::WatAST;
use super::error::UnresolvedReference;
use wat_macros::wat_special_form_impl;

/// Scan top-level forms for `(:wat::core::use! :rust::...)` and record
/// them in `use_decls`. Emits an `UnresolvedReference` if the requested
/// symbol isn't in the build-time rust-deps registry.
///
/// Arc 255 Stone 1a-ε — the `role = declare` pointer for `:wat::core::use!` (STOP-4 measured
/// FALSE for this row: a real freeze-time processor exists — this fn, run as Pass 1 of
/// `resolve_references`, called from `freeze.rs` step 7, strictly before evaluation). See
/// `src/intrinsic/special/use_form.rs`'s module doc for the full finding.
#[wat_special_form_impl(":wat::core::use!", role = declare)]
pub(super) fn collect_use_declarations(
    form: &WatAST,
    registry: &crate::rust_deps::RustDepsRegistry,
    use_decls: &mut crate::rust_deps::UseDeclarations,
    unresolved: &mut Vec<UnresolvedReference>,
) {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(head, head_span)) = items.first() {
            if head == ":wat::core::use!" {
                // Expect exactly one keyword argument.
                if items.len() != 2 {
                    unresolved.push(UnresolvedReference {
                        path: head.clone(),
                        context:
                            "(:wat::core::use! :rust::Path) expects exactly one keyword argument",
                        span: head_span.clone(),
                    });
                    return;
                }
                if let WatAST::Keyword(path, path_span) = &items[1] {
                    if !registry.has_type(path) {
                        unresolved.push(UnresolvedReference {
                            path: path.clone(),
                            context: "rust symbol not available in wat; declare it via its shim",
                            span: path_span.clone(),
                        });
                        return;
                    }
                    use_decls.declare(path.clone());
                } else {
                    unresolved.push(UnresolvedReference {
                        path: head.clone(),
                        context: "(:wat::core::use! ...) argument must be a keyword path",
                        span: head_span.clone(),
                    });
                }
            }
        }
    }
}
