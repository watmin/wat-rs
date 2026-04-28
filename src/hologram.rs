//! Arc 074 slice 1 — `Hologram`: coordinate-cell store with
//! cosine readout, unbounded. HolonAST → HolonAST.
//!
//! The substrate's natural neighborhoods (`floor(sqrt(d))` cells per
//! d) are the cache's pre-filter: ASTs in different cells live in
//! different regions of the algebra grid and can't satisfy the same
//! query. `put` indexes by user-supplied `pos: f64` (normalized to
//! `[0, 100]`); `get` walks the two adjacent cells around the probe's
//! position and runs cosine readout against each candidate.
//!
//! HolonAST → HolonAST. The val type is fixed (both sides are forms in
//! the algebra). Consumers needing HolonAST → other-shape (e.g., form →
//! Vector for an encode cache) reach for a different primitive — the
//! coordinate-cell + cosine-readout shape only earns its keep when both
//! sides of the lookup are forms in the algebra.
//!
//! Unbounded — entries never evict. The bounded variant (`HologramLRU`)
//! ships in slice 2 as a sibling crate composing this primitive plus
//! `wat-lru`'s LRU. Per arc 074 DESIGN: two concrete types, no trait;
//! polymorphism via enum-wrapping (058-030).
//!
//! Per `ZERO-MUTEX.md` Tier 2: thread-owned mutable. The wat-side
//! Value variant wraps this in `ThreadOwnedCell` for scope safety
//! with zero Mutex.

use crate::runtime::RuntimeError;
use holon::HolonAST;
use std::collections::HashMap;

/// Population-keyed coordinate cell store. Unbounded; entries persist
/// until the store is dropped.
pub struct Hologram {
    /// Outer length is `num_cells = floor(sqrt(d))`. Each cell is a
    /// HashMap of `(key → val)` pairs (both HolonAST) whose pos landed
    /// in that neighborhood.
    cells: Vec<HashMap<HolonAST, HolonAST>>,
    /// Cached `floor(sqrt(d))` so we don't recompute on every op.
    num_cells: usize,
}

impl Hologram {
    /// Construct an empty store sized for the given encoding `d`.
    /// `num_cells = floor(sqrt(d))`. Caller passes the same d that
    /// the dim router will route forms at; substrate-internal users
    /// typically read `DEFAULT_TIERS[0]` (10000 → 100 cells).
    pub fn new(d: usize) -> Self {
        let num_cells = (d as f64).sqrt().floor() as usize;
        // Ensure at least one cell — degenerate d=0/1 still produces a
        // usable (if useless) store. Not a panic — `new` shouldn't fail
        // on wonky d.
        let num_cells = num_cells.max(1);
        let cells = (0..num_cells).map(|_| HashMap::new()).collect();
        Hologram { cells, num_cells }
    }

    /// `num_cells` for this store. Read-only; matches `floor(sqrt(d))`
    /// of the d passed to `new`.
    pub fn num_cells(&self) -> usize {
        self.num_cells
    }

    /// Total entries across all cells.
    pub fn len(&self) -> usize {
        self.cells.iter().map(|c| c.len()).sum()
    }

    /// `cells[idx]` — read access for the eval-side cosine loop. Out
    /// of bounds is a substrate bug; panic-friendly diagnostic.
    pub fn cell(&self, idx: usize) -> &HashMap<HolonAST, HolonAST> {
        &self.cells[idx]
    }

    /// Insert `(key, val)` at the cell determined by `pos`. Pos is
    /// pre-validated; this method assumes the caller has run
    /// [`pos_to_cell_index`].
    ///
    /// HashMap semantics: existing key gets overwritten. The store is
    /// idempotent at the same `(pos, key)`.
    pub fn put_at_index(&mut self, idx: usize, key: HolonAST, val: HolonAST) {
        self.cells[idx].insert(key, val);
    }
}

/// Map a user-supplied `pos: f64` (normalized to `[0, 100]`) into a
/// cell index for a store with `num_cells` cells.
///
/// Strict validation per arc 074 DESIGN Q4: `pos < 0`, `pos > 100`,
/// or NaN raises `RuntimeError::MalformedForm`. No silent clamping.
/// Callers play by the rules.
pub fn pos_to_cell_index(
    op: &str,
    pos: f64,
    num_cells: usize,
) -> Result<usize, RuntimeError> {
    if pos.is_nan() {
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: "pos must be finite in [0, 100]; got NaN".into(),
        });
    }
    if !(0.0..=100.0).contains(&pos) {
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!("pos must be in [0, 100]; got {}", pos),
        });
    }
    let raw = (pos * num_cells as f64 / 100.0).floor() as usize;
    // pos == 100.0 lands on raw == num_cells (just past the last index);
    // clamp to num_cells - 1 so the highest cell remains addressable.
    Ok(raw.min(num_cells.saturating_sub(1)))
}

/// `get`-side spread: returns `(left, right)` cell indices to walk
/// for a probe at `pos`. When `pos` falls cleanly inside a cell,
/// `left == right`. When `pos` lies on a cell boundary, `left + 1 == right`.
/// Out-of-range pos validates the same as [`pos_to_cell_index`].
pub fn pos_to_cell_spread(
    op: &str,
    pos: f64,
    num_cells: usize,
) -> Result<(usize, usize), RuntimeError> {
    if pos.is_nan() {
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: "pos must be finite in [0, 100]; got NaN".into(),
        });
    }
    if !(0.0..=100.0).contains(&pos) {
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!("pos must be in [0, 100]; got {}", pos),
        });
    }
    let scaled = pos * num_cells as f64 / 100.0;
    let max_idx = num_cells.saturating_sub(1);
    let left = (scaled.floor() as usize).min(max_idx);
    let right = (scaled.ceil() as usize).min(max_idx);
    Ok((left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_d_10000_yields_100_cells() {
        let h = Hologram::new(10000);
        assert_eq!(h.num_cells(), 100);
    }

    #[test]
    fn new_d_4096_yields_64_cells() {
        let h = Hologram::new(4096);
        assert_eq!(h.num_cells(), 64);
    }

    #[test]
    fn new_d_1024_yields_32_cells() {
        let h = Hologram::new(1024);
        assert_eq!(h.num_cells(), 32);
    }

    #[test]
    fn pos_to_index_at_d_10000() {
        let n = 100;
        assert_eq!(pos_to_cell_index("test", 0.0, n).unwrap(), 0);
        assert_eq!(pos_to_cell_index("test", 1.43, n).unwrap(), 1);
        assert_eq!(pos_to_cell_index("test", 50.0, n).unwrap(), 50);
        assert_eq!(pos_to_cell_index("test", 99.999, n).unwrap(), 99);
        assert_eq!(pos_to_cell_index("test", 100.0, n).unwrap(), 99);
    }

    #[test]
    fn pos_to_index_at_d_4096() {
        let n = 64;
        // pos=50 at 64 cells → floor(50 * 64 / 100) = floor(32) = 32
        assert_eq!(pos_to_cell_index("test", 50.0, n).unwrap(), 32);
        assert_eq!(pos_to_cell_index("test", 100.0, n).unwrap(), 63);
    }

    #[test]
    fn pos_validation_rejects_negative() {
        let err = pos_to_cell_index("test", -0.1, 100).unwrap_err();
        match err {
            RuntimeError::MalformedForm { reason, .. } => assert!(reason.contains("-0.1")),
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn pos_validation_rejects_above_100() {
        let err = pos_to_cell_index("test", 100.001, 100).unwrap_err();
        match err {
            RuntimeError::MalformedForm { reason, .. } => assert!(reason.contains("100.001")),
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn pos_validation_rejects_nan() {
        let err = pos_to_cell_index("test", f64::NAN, 100).unwrap_err();
        match err {
            RuntimeError::MalformedForm { reason, .. } => assert!(reason.contains("NaN")),
            other => panic!("expected MalformedForm, got {:?}", other),
        }
    }

    #[test]
    fn spread_at_cell_boundary() {
        // pos=2.0 at 100 cells → scaled=2.0; floor=2, ceil=2 → (2, 2)
        // pos=2.5 → scaled=2.5; floor=2, ceil=3 → (2, 3)
        let (l, r) = pos_to_cell_spread("test", 2.0, 100).unwrap();
        assert_eq!((l, r), (2, 2));
        let (l, r) = pos_to_cell_spread("test", 2.5, 100).unwrap();
        assert_eq!((l, r), (2, 3));
    }

    #[test]
    fn spread_clamps_at_boundary() {
        let (l, r) = pos_to_cell_spread("test", 100.0, 100).unwrap();
        assert_eq!((l, r), (99, 99));
    }

    #[test]
    fn put_and_len_track_inserts_across_cells() {
        let mut h = Hologram::new(10000);
        h.put_at_index(5, HolonAST::keyword("k1"), HolonAST::keyword("v1"));
        h.put_at_index(50, HolonAST::keyword("k2"), HolonAST::keyword("v2"));
        assert_eq!(h.len(), 2);
        assert_eq!(h.cell(5).len(), 1);
        assert_eq!(h.cell(50).len(), 1);
        assert_eq!(h.cell(99).len(), 0);
    }

    #[test]
    fn put_idempotent_at_same_key() {
        let mut h = Hologram::new(10000);
        let key = HolonAST::keyword("k1");
        h.put_at_index(5, key.clone(), HolonAST::keyword("a"));
        h.put_at_index(5, key, HolonAST::keyword("b"));
        assert_eq!(h.len(), 1);
    }
}
