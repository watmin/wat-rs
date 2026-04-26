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
use holon::HolonAST;
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
/// construction. Contract: **HolonAST in, `Option<usize>` out** —
/// "what dimension is best for this AST's surface?" `None` signals
/// no tier fits; caller dispatches per `capacity-mode`.
///
/// The router sees the whole AST; it can measure whatever it wants
/// about it. The built-in [`SizingRouter`] looks at surface arity
/// (the top-level cardinality); user routers via
/// [`WatLambdaRouter`] can inspect deeper.
///
/// `sym` is passed so user-supplied routers (wat lambdas) can
/// evaluate against the frozen symbol table. The built-in
/// [`SizingRouter`] ignores it.
pub trait DimRouter: Send + Sync + fmt::Debug {
    /// Return the smallest dim `d` such that this router accepts
    /// the given AST's construction. `None` signals overflow.
    fn pick(&self, ast: &HolonAST, sym: &SymbolTable) -> Option<usize>;
}

/// Top-level cardinality of a [`HolonAST`] — the "surface-deep"
/// count used by the default router. Each variant has a fixed
/// shape except Bundle which is variable.
pub fn immediate_arity(ast: &HolonAST) -> usize {
    match ast {
        // Primitive leaves are atomic (arity 1).
        HolonAST::Symbol(_)
        | HolonAST::String(_)
        | HolonAST::I64(_)
        | HolonAST::F64(_)
        | HolonAST::Bool(_) => 1,
        HolonAST::Atom(_) => 1,
        HolonAST::Bind(_, _) => 2,
        HolonAST::Bundle(children) => children.len(),
        HolonAST::Permute(_, _) => 1,
        HolonAST::Thermometer { .. } => 1,
        HolonAST::Blend(_, _, _, _) => 2,
    }
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
    fn pick(&self, ast: &HolonAST, _sym: &SymbolTable) -> Option<usize> {
        // Surface-deep: look at the AST's top-level cardinality.
        // Smallest tier d where sqrt(d) >= immediate_size, i.e.,
        // d >= immediate_size².
        let size = immediate_arity(ast);
        let needed = size.checked_mul(size)?;
        self.tiers
            .iter()
            .find(|&&d| d >= needed)
            .copied()
    }
}

/// User-supplied dim router. Wraps a wat function of signature
/// `:fn(:wat::holon::HolonAST) -> :Option<i64>` registered via
/// `(:wat::config::set-dim-router! <expr>)`. At pick time the wat
/// function is invoked through [`apply_function`] with the AST
/// wrapped as `Value::holon__HolonAST`; the returned
/// `Value::Option` is mapped to `Option<usize>`.
///
/// The router receives the whole AST; it can measure surface arity
/// via `(:wat::holon::statement-length ast)`, switch on AST variant
/// kind, or do deeper introspection. The decision is entirely the
/// user's.
///
/// Any runtime error during invocation (eval failure, return-type
/// mismatch, non-positive dim, etc.) surfaces as `None`. The caller
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
    fn pick(&self, ast: &HolonAST, sym: &SymbolTable) -> Option<usize> {
        let arg = Value::holon__HolonAST(Arc::new(ast.clone()));
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

/// Ambient sigma function. `sigma_at(d, sym)` returns the sigma
/// count (unitless σ-multiplier) at dim `d`. Used by presence? /
/// coincident? to compute their per-d floors:
/// `floor_at_d = sigma_at(d) / sqrt(d)`.
///
/// Two ambient slots on SymbolTable — one for presence, one for
/// coincident. Built-in defaults ship as Rust impls
/// ([`DefaultPresenceSigma`], [`DefaultCoincidentSigma`]); user
/// overrides via `set-presence-sigma!` / `set-coincident-sigma!`
/// wrap a wat function in [`WatLambdaSigmaFn`].
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
/// coincident? remain meaningful even if the user's router misfires.
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

    /// Build a Bundle of `n` opaque atom payloads — the default
    /// router only reads immediate arity so the payloads don't
    /// matter, just the count.
    fn bundle_of(n: usize) -> HolonAST {
        let children: Vec<HolonAST> = (0..n)
            .map(|i| HolonAST::string(format!("a-{}", i)))
            .collect();
        HolonAST::bundle(children)
    }

    #[test]
    fn default_router_picks_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        // 16-item bundle → sqrt(256)=16 → d=256.
        assert_eq!(r.pick(&bundle_of(16), &s), Some(256));
        // 17 → 17² = 289 > 256 → d=4096.
        assert_eq!(r.pick(&bundle_of(17), &s), Some(4096));
        // 64 → 64² = 4096 exactly → d=4096.
        assert_eq!(r.pick(&bundle_of(64), &s), Some(4096));
        // 65 → 4225 > 4096 → d=10000.
        assert_eq!(r.pick(&bundle_of(65), &s), Some(10000));
        // 100 → 10000 exactly → d=10000.
        assert_eq!(r.pick(&bundle_of(100), &s), Some(10000));
        // 101 → 10201 > 10000 → d=100000.
        assert_eq!(r.pick(&bundle_of(101), &s), Some(100000));
        // 316 → 99856 < 100000 → d=100000.
        assert_eq!(r.pick(&bundle_of(316), &s), Some(100000));
        // 317 → 100489 > 100000 → overflow.
        assert_eq!(r.pick(&bundle_of(317), &s), None);
    }

    #[test]
    fn leaf_atom_fits_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        // An Atom has immediate arity 1 → smallest tier fits.
        assert_eq!(r.pick(&HolonAST::string("alice"), &s), Some(256));
        // Empty bundle (arity 0) also fits smallest.
        assert_eq!(r.pick(&bundle_of(0), &s), Some(256));
    }

    #[test]
    fn custom_single_tier_matches_legacy_behavior() {
        let r = SizingRouter::with_tiers(vec![10000]);
        let s = sym();
        assert_eq!(r.pick(&HolonAST::string("x"), &s), Some(10000));
        assert_eq!(r.pick(&bundle_of(100), &s), Some(10000));
        assert_eq!(r.pick(&bundle_of(101), &s), None);
    }

    #[test]
    fn overflow_past_largest_tier_is_none() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        assert_eq!(r.pick(&bundle_of(317), &s), None);
        assert_eq!(r.pick(&bundle_of(1000), &s), None);
    }

    #[test]
    fn empty_tier_list_always_overflows() {
        let r = SizingRouter::with_tiers(vec![]);
        let s = sym();
        assert_eq!(r.pick(&bundle_of(0), &s), None);
        assert_eq!(r.pick(&HolonAST::string("x"), &s), None);
        assert_eq!(r.pick(&bundle_of(100), &s), None);
    }

    #[test]
    fn bind_arity_is_2() {
        let r = SizingRouter::with_default_tiers();
        let s = sym();
        let ast = HolonAST::bind(HolonAST::string("a"), HolonAST::string("b"));
        // arity 2 → fits smallest tier (sqrt(256)=16 ≥ 2).
        assert_eq!(r.pick(&ast, &s), Some(256));
    }
}
