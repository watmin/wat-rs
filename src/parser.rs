//! S-expression parser — tokens → `WatAST`.
//!
//! Recursive descent over the s-expression grammar, dispatching on head
//! keyword (`:wat/core/define` → `Define` variant, `:wat/algebra/...` →
//! `UpperCall`, etc.). Produces structured errors with source position.
//!
//! This module is a stub until the parser task lands.
