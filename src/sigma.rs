//! Sigma functions — ambient runtime capability used by `presence?`
//! and `coincident?` to compute their per-d floors:
//! `floor_at_d = sigma_at(d) / sqrt(d)`.
//!
//! Two ambient slots on `SymbolTable` — one for presence, one for
//! coincident. Built-in defaults ship as Rust impls
//! ([`DefaultPresenceSigma`], [`DefaultCoincidentSigma`]); user
//! overrides via `(:wat::config::set-presence-sigma!)` /
//! `(:wat::config::set-coincident-sigma!)` wrap a wat function in
//! [`WatLambdaSigmaFn`].
//!
//! Arc 077: this file used to be `dim_router.rs` and carried the
//! per-form router infrastructure (`DimRouter`, `SizingRouter`,
//! `WatLambdaRouter`, `immediate_arity`, `DEFAULT_TIERS`). The router
//! was retired when arc 067 collapsed `DEFAULT_TIERS` to `[10000]`
//! and arc 076's therm-routed Hologram made it clear that the
//! one-d-per-program model is what we actually wanted. The router
//! types are gone; only the sigma family — which is genuinely a
//! per-d math knob, not a routing concept — survives.

use crate::runtime::{apply_function, Function, SymbolTable, Value};
use crate::span::Span;
use std::fmt;
use std::sync::Arc;

/// Ambient sigma function. `sigma_at(d, sym)` returns the sigma
/// count (unitless σ-multiplier) at dim `d`. Used by `presence?` /
/// `coincident?` to compute their per-d floors.
pub trait SigmaFn: Send + Sync + fmt::Debug {
    fn sigma_at(&self, d: usize, sym: &SymbolTable) -> i64;
}

/// Built-in default presence sigma: arc 024's `floor(sqrt(d)/2) - 1`
/// formula, clamped ≥ 1 so degenerate tiers stay meaningful.
/// At d=10k → 49, d=1024 → 15, d=256 → 7, d=16 → 1.
#[derive(Clone, Debug)]
pub struct DefaultPresenceSigma;

impl SigmaFn for DefaultPresenceSigma {
    fn sigma_at(&self, d: usize, _sym: &SymbolTable) -> i64 {
        let sqrt_d = (d as f64).sqrt();
        let s = (sqrt_d.floor() / 2.0).floor() as i64 - 1;
        s.max(1)
    }
}

/// Built-in default coincident sigma: 1 (1σ native granularity —
/// the smallest cosine distance the substrate can physically
/// distinguish at any d).
#[derive(Clone, Debug)]
pub struct DefaultCoincidentSigma;

impl SigmaFn for DefaultCoincidentSigma {
    fn sigma_at(&self, _d: usize, _sym: &SymbolTable) -> i64 {
        1
    }
}

/// User-supplied sigma function. Wraps a wat function of signature
/// `:fn(:i64) -> :i64`. At `sigma_at(d, sym)` the wat function is
/// invoked with `Value::i64(d)`; the returned `Value::i64` is
/// returned as the sigma count. Any runtime error or shape mismatch
/// folds to 1 — the minimum geometric σ — so presence? /
/// coincident? remain meaningful even if the user's lambda misfires.
pub struct WatLambdaSigmaFn {
    pub path: String,
    pub func: Arc<Function>,
}

impl fmt::Debug for WatLambdaSigmaFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WatLambdaSigmaFn")
            .field("path", &self.path)
            .finish()
    }
}

impl SigmaFn for WatLambdaSigmaFn {
    fn sigma_at(&self, d: usize, sym: &SymbolTable) -> i64 {
        let arg = Value::i64(d as i64);
        let call_span = Span::unknown();
        let result = apply_function(Arc::clone(&self.func), vec![arg], sym, call_span);
        match result {
            Ok(Value::i64(n)) if n >= 1 => n,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym() -> SymbolTable {
        SymbolTable::new()
    }

    #[test]
    fn default_presence_sigma_clamps_at_one_for_small_d() {
        let s = DefaultPresenceSigma;
        let table = sym();
        assert_eq!(s.sigma_at(16, &table), 1);
        assert_eq!(s.sigma_at(64, &table), 3);
        assert_eq!(s.sigma_at(256, &table), 7);
        assert_eq!(s.sigma_at(1024, &table), 15);
        assert_eq!(s.sigma_at(10000, &table), 49);
    }

    #[test]
    fn default_coincident_sigma_is_constant_one() {
        let s = DefaultCoincidentSigma;
        let table = sym();
        assert_eq!(s.sigma_at(16, &table), 1);
        assert_eq!(s.sigma_at(10000, &table), 1);
    }
}
