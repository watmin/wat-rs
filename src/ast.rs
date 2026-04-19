//! `WatAST` — the language-surface AST the parser produces.
//!
//! Distinct from `holon::HolonAST`. WatAST carries every form the grammar
//! admits at the language level (`define`, `lambda`, `struct`, `load!`,
//! `let`, `if`, `match`, `defmacro`, …). Algebra-core calls appear as
//! `UpperCall` nodes that are lowered to `holon::HolonAST` at evaluation
//! time via the [`crate::lower`] module.
//!
//! This module is a stub — variants are added as Phase 1 work lands.

/// Placeholder for the parsed source tree. Variants will be filled in by
/// the WatAST type-definition task.
#[derive(Debug)]
pub enum WatAST {
    /// Placeholder variant to keep the crate compiling during bootstrap.
    /// Removed once real variants are added.
    Placeholder,
}
