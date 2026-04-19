//! WatAST → HolonAST lowering.
//!
//! Evaluator path for algebra-core `UpperCall` subtrees. Converts
//! `(:wat/algebra/Atom x)` into `holon::HolonAST::atom(x)`, and similarly
//! for `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend`. Literal args
//! lower to the corresponding Rust primitive (`i64` / `f64` / `String` /
//! etc.); keyword args lower via `HolonAST::keyword`.
//!
//! This module is a stub until the lowering task lands.
