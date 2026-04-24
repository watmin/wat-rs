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
//! User override ships in a later slice (arc 037 slice 5) via the
//! `set-dim-router!` primitive — the config carries the user's wat
//! lambda; substrate invokes it at construction time. For now the
//! default is the only router; it attaches to
//! [`crate::runtime::SymbolTable`]'s capability slot at freeze.

use std::fmt;

/// Opinionated default tier list: `[256, 4096, 10000, 100000]`.
///
/// At d=256 the bundle capacity is 16 items; at d=4096 it's 64; at
/// d=10000 it's 100; at d=100000 it's ~316. Four orders of magnitude
/// of statement-richness coverage in four tiers. User override
/// available via [`SizingRouter::with_tiers`] or, eventually, a wat
/// lambda through the `set-dim-router!` primitive.
pub const DEFAULT_TIERS: &[usize] = &[256, 4096, 10000, 100000];

/// Ambient runtime capability that picks vector dimension per
/// construction. `pick(n)` returns `Some d` when a tier fits item
/// count `n`, or `None` when all tiers overflow.
pub trait DimRouter: Send + Sync + fmt::Debug {
    /// Return the smallest dim `d` such that this router accepts a
    /// construction of `immediate_size` items. `None` signals
    /// overflow — caller dispatches per `capacity-mode`.
    fn pick(&self, immediate_size: usize) -> Option<usize>;
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
    fn pick(&self, immediate_size: usize) -> Option<usize> {
        // Smallest tier d where sqrt(d) >= immediate_size.
        // Equivalent integer form: d >= immediate_size².
        let needed = immediate_size.checked_mul(immediate_size)?;
        self.tiers
            .iter()
            .find(|&&d| d >= needed)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_router_picks_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        // 16 items → sqrt(256)=16 fits exactly → d=256.
        assert_eq!(r.pick(16), Some(256));
        // 17 items → 17² = 289 > 256, needs 4096 → d=4096.
        assert_eq!(r.pick(17), Some(4096));
        // 64 items → 64² = 4096 exactly → d=4096.
        assert_eq!(r.pick(64), Some(4096));
        // 65 items → 65² = 4225 > 4096, needs 10000 → d=10000.
        assert_eq!(r.pick(65), Some(10000));
        // 100 items → 100² = 10000 exactly → d=10000.
        assert_eq!(r.pick(100), Some(10000));
        // 101 items → 101² = 10201 > 10000, needs 100000 → d=100000.
        assert_eq!(r.pick(101), Some(100000));
        // 316 items → 316² = 99856 < 100000 → d=100000.
        assert_eq!(r.pick(316), Some(100000));
        // 317 items → 317² = 100489 > 100000 → overflow.
        assert_eq!(r.pick(317), None);
    }

    #[test]
    fn leaf_item_fits_smallest_tier() {
        let r = SizingRouter::with_default_tiers();
        // A single atom (N=1) fits anywhere; router picks smallest.
        assert_eq!(r.pick(1), Some(256));
        // Zero items (empty bundle) also fits smallest.
        assert_eq!(r.pick(0), Some(256));
    }

    #[test]
    fn custom_single_tier_matches_legacy_behavior() {
        // User-supplied single-tier list reproduces pre-arc-037
        // single-dim behavior.
        let r = SizingRouter::with_tiers(vec![10000]);
        assert_eq!(r.pick(1), Some(10000));
        assert_eq!(r.pick(100), Some(10000));
        // Past the single tier: overflow.
        assert_eq!(r.pick(101), None);
    }

    #[test]
    fn overflow_past_largest_tier_is_none() {
        let r = SizingRouter::with_default_tiers();
        // sqrt(100000) ≈ 316.2 — 317+ items overflow.
        assert_eq!(r.pick(317), None);
        assert_eq!(r.pick(1000), None);
        assert_eq!(r.pick(usize::MAX), None);
    }

    #[test]
    fn empty_tier_list_always_overflows() {
        let r = SizingRouter::with_tiers(vec![]);
        assert_eq!(r.pick(0), None);
        assert_eq!(r.pick(1), None);
        assert_eq!(r.pick(100), None);
    }
}
