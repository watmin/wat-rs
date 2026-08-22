//! Collection of `:wat::core::use!` declarations and rust-deps coverage checks.
//!
//! Pass 1 of [`super::resolve_references`]: scan top-level forms for
//! `(:wat::core::use! :rust::...)` and validate each against the build-time
//! rust-deps registry.

use crate::ast::WatAST;
use super::error::UnresolvedReference;

/// Scan top-level forms for `(:wat::core::use! :rust::...)` and record
/// them in `use_decls`. Emits an `UnresolvedReference` if the requested
/// symbol isn't in the build-time rust-deps registry.
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
