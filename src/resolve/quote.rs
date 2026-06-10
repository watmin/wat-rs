//! Quasiquote/quote boundary descent for the name-resolution pass.
//!
//! Quote-family forms capture their arguments as AST data; only
//! `:wat::core::unquote` and `:wat::core::unquote-splicing` escapes
//! contain live code that must be walked for call-head resolution.

use crate::ast::WatAST;
use crate::macros::MacroRegistry;
use crate::runtime::SymbolTable;
use super::boundary::is_unquote_escape;
use super::error::UnresolvedReference;
use super::walk::check_form;

/// Walk a quasiquote template, resolving call heads only inside
/// `:wat::core::unquote` and `:wat::core::unquote-splicing` escape forms.
///
/// Everything else in the template is data and must not be descended into.
/// Nested `(:wat::core::quasiquote ...)` inside the template is also treated
/// as opaque data (out of scope for Gap F-2; see note in `check_form`).
pub(super) fn check_quasiquote_template(
    node: &WatAST,
    sym: &SymbolTable,
    macros: &MacroRegistry,
    use_decls: &crate::rust_deps::UseDeclarations,
    unresolved: &mut Vec<UnresolvedReference>,
) {
    // Only List forms can be unquote/unquote-splicing escapes; check the
    // head keyword for those special forms first.
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_unquote_escape(head) {
                // Escape: the argument is live code. Use normal check_form.
                for arg in items.iter().skip(1) {
                    check_form(arg, sym, macros, use_decls, unresolved);
                }
                return;
            }
            // Any other list form (including nested quasiquote) is template
            // data — don't flag the call head, but DO recurse into children
            // looking for unquote/unquote-splicing escapes deeper in the tree.
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector, Map,
    // and Set uniformly. Walkers that only recurse into List silently
    // miss unquote escapes inside bracketed forms (e.g. let-binding vectors).
    // children() returns &[] for leaf nodes so this is a no-op for atoms.
    for child in node.children().iter() {
        check_quasiquote_template(child, sym, macros, use_decls, unresolved);
    }
    // Atoms (symbols, keywords, literals): children() → &[]; loop is a no-op.
}
