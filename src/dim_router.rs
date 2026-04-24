//! Dim router — ambient runtime capability that decides vector
//! dimension per Atom/Bundle construction.
//!
//! Replaces the pre-arc-037 single-dim consultation
//! (`ctx.config.dims`) with a function-per-shape: each construction
//! site queries the router with its immediate item count; the router
//! returns `Some d` when a tier fits and `None` when all tiers
//! overflow (caller dispatches per `capacity-mode`).
//!
//! The built-in default is [`SizingRouter`] with [`DEFAULT_TIERS`].
//! It picks the smallest tier `d` in the list whose `sqrt(d)` is at
//! least the immediate item count — the Kanerva capacity bound
//! (BOOK Ch 41: `d = K²` where K is max statement size).
//!
//! User override via [`WatLambdaRouter`]: the user writes a wat
//! function `fn(:i64) -> :Option<:i64>` and registers it via
//! `(:wat::config::set-dim-router! :my::router)`. Freeze looks up
//! the function and installs a WatLambdaRouter that invokes it at
//! pick time. Default is [`SizingRouter::with_default_tiers`] when
//! no user router is set.

use crate::runtime::{apply_function, Function, RuntimeError, SymbolTable, Value};
use crate::span::Span;
use std::fmt;
use std::sync::Arc;

/// Opinionated default tier list: `[256, 4096, 10000, 100000]`.
///
/// At d=256 the bundle capacity is 16 items; at d=4096 it's 64; at
/// d=10000 it's 100; at d=100000 it's ~316. Four orders of magnitude
/// of statement-richness coverage in four tiers. User override
/// available via [`SizingRouter::with_tiers`] or, eventually, a wat
/// lambda through the `set-dim-router!` primitive.
pub const DEFAULT_TIERS: &[usize] = &[256, 4096, 10000, 100000];

/// Ambient runtime capability that picks vector dimension per
/// construction. `pick(n, sym)` returns `Some d` when a tier fits
/// item count `n`, or `None` when all tiers overflow.
///
/// `sym` is passed so user-supplied routers (wat lambdas) can
/// evaluate against the frozen symbol table. The built-in
/// [`SizingRouter`] ignores it.
pub trait DimRouter: Send + Sync + fmt::Debug {
    /// Return the smallest dim `d` such that this router accepts a
    /// construction of `immediate_size` items. `None` signals
    /// overflow — caller dispatches per `capacity-mode`.
    fn pick(&self, immediate_size: usize, sym: &SymbolTable) -> Option<usize>;
}

/// The built-in sizing function. Closes over a tier list; picks the
/// smallest tier whose `sqrt(d) ≥ immediate_size`.
#[derive(Clone)]
pub struct SizingRouter {
    tiers: Vec<usize>,
}

impl SizingRouter {
    /// Build a sizing router with [`DEFAULT_TIERS`]. This is the
    /// substrate default attached by freeze.
    pub fn with_default_tiers() -> Self {
        Self {
            tiers: DEFAULT_TIERS.to_vec(),
        }
    }

    /// Build a sizing router with an arbitrary tier list. Tiers
    /// should be positive and sorted ascending; unsorted lists still
    /// work but pick the first tier in iteration order that fits,
    /// which may not be the smallest.
    pub fn with_tiers(tiers: Vec<usize>) -> Self {
        Self { tiers }
    }

    /// The tier list this router considers.
    pub fn tiers(&self) -> &[usize] {
        &self.tiers
    }
}

impl fmt::Debug for SizingRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SizingRouter")
            .field("tiers", &self.tiers)
            .finish()
    }
}

impl DimRouter for SizingRouter {
    fn pick(&self, immediate_size: usize, _sym: &SymbolTable) -> Option<usize> {
        // Smallest tier d where sqrt(d) >= immediate_size.
        // Equivalent integer form: d >= immediate_size².
        let needed = immediate_size.checked_mul(immediate_size)?;
        self.tiers
            .iter()
            .find(|&&d| d >= needed)
            .copied()
    }
}

/// User-supplied dim router. Wraps a wat function of signature
/// `:fn(:i64) -> :Option<:i64>` registered via
/// `(:wat::config::set-dim-router! :my::router)`. At pick time the
/// wat function is invoked through [`apply_function`] with the
/// current immediate size; the returned `Value::Option` is mapped to
/// `Option<usize>`.
///
/// Any runtime error during invocation (eval failure, return-type
/// mismatch, negative dim, etc.) surfaces as `None`. The caller
/// then dispatches per `capacity-mode` — user router bugs become
/// user-visible overflows rather than silent corruption.
pub struct WatLambdaRouter {
    pub path: String,
    pub func: Arc<Function>,
}

impl fmt::Debug for WatLambdaRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WatLambdaRouter")
            .field("path", &self.path)
            .finish()
    }
}

impl DimRouter for WatLambdaRouter {
    fn pick(&self, immediate_size: usize, sym: &SymbolTable) -> Option<usize> {
        let arg = Value::i64(immediate_size as i64);
        let call_span = Span::unknown();
        let result = apply_function(Arc::clone(&self.func), vec![arg], sym, call_span);
        match result {
            Ok(Value::Option(opt_arc)) => match &*opt_arc {
                Some(Value::i64(n)) if *n > 0 => Some(*n as usize),
                _ => None,
            },
            Ok(_) | Err(RuntimeError::TailCall { .. } | RuntimeError::TryPropagate(..)) => None,
            Err(_) => None,
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
    fn default_router_picks_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        // 16 items → sqrt(256)=16 fits exactly → d=256.
        assert_eq!(r.pick(16, &s), Some(256));
        // 17 items → 17² = 289 > 256, needs 4096 → d=4096.
        assert_eq!(r.pick(17, &s), Some(4096));
        // 64 items → 64² = 4096 exactly → d=4096.
        assert_eq!(r.pick(64, &s), Some(4096));
        // 65 items → 65² = 4225 > 4096, needs 10000 → d=10000.
        assert_eq!(r.pick(65, &s), Some(10000));
        // 100 items → 100² = 10000 exactly → d=10000.
        assert_eq!(r.pick(100, &s), Some(10000));
        // 101 items → 101² = 10201 > 10000, needs 100000 → d=100000.
        assert_eq!(r.pick(101, &s), Some(100000));
        // 316 items → 316² = 99856 < 100000 → d=100000.
        assert_eq!(r.pick(316, &s), Some(100000));
        // 317 items → 317² = 100489 > 100000 → overflow.
        assert_eq!(r.pick(317, &s), None);
    }

    #[test]
    fn leaf_item_fits_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        assert_eq!(r.pick(1, &s), Some(256));
        assert_eq!(r.pick(0, &s), Some(256));
    }

    #[test]
    fn custom_single_tier_matches_legacy_behavior() {
        let r = SizingRouter::with_tiers(vec![10000]);
        let s = sym();
        assert_eq!(r.pick(1, &s), Some(10000));
        assert_eq!(r.pick(100, &s), Some(10000));
        assert_eq!(r.pick(101, &s), None);
    }

    #[test]
    fn overflow_past_largest_tier_is_none() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        assert_eq!(r.pick(317, &s), None);
        assert_eq!(r.pick(1000, &s), None);
        assert_eq!(r.pick(usize::MAX, &s), None);
    }

    #[test]
    fn empty_tier_list_always_overflows() {
        let r = SizingRouter::with_tiers(vec![]);
        let s = sym();
        assert_eq!(r.pick(0, &s), None);
        assert_eq!(r.pick(1, &s), None);
        assert_eq!(r.pick(100, &s), None);
    }
}
