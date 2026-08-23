//! `PVec` — the promoting vector: an array from bulk build, an RRB tree after persistent conj.
//!
//! `:wat::core::PersistentVector` was an unconditional `rpds::VectorSync`, so every freeze of
//! N elements paid N RRB `push_back_mut` (out:query V−C **3.36 ms** / 40k). One representation
//! chosen globally is a claim about how vectors are built; promoting per instance makes no claim
//! — bulk `from_vec` is a contiguous array at any length; persistent `conj` of a large array
//! promotes so wat-level conj stays O(log n).
//!
//! ## The wall — representation must be UNOBSERVABLE
//!
//! Promotion is **one-way**: `push_back` past [`PROMOTION_THRESHOLD`] promotes; nothing demotes.
//! Two vectors with the same elements in the same order are the same value, whichever arm holds
//! them. `PartialEq` / `Hash` compare the sequence, never the container. Get this wrong and a
//! vector used as a map key silently misses.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::value::Value;

/// Persistent `conj` of an Array this long promotes to the RRB tree so incremental
/// wat-level `conj` stays O(log n). Bulk `from_vec` ignores the cap.
pub const PROMOTION_THRESHOLD: usize = 8;

/// A persistent vector that picks its representation from how it was built.
#[derive(Debug, Clone)]
pub enum PVec {
    /// Bulk `from_vec` / unique `push_back_mut`. Index is a slice. Any length.
    Array(Arc<Vec<Value>>),
    /// After persistent `push_back` past the threshold — the prior representation.
    Tree(rpds::VectorSync<Value>),
}

impl Default for PVec {
    fn default() -> Self {
        PVec::new()
    }
}

impl PVec {
    pub fn new() -> Self {
        PVec::Array(Arc::new(Vec::new()))
    }

    /// Bulk build — Array arm, any length. The freeze intern
    /// (`DESIGN-STONE-promoting-vector`).
    pub fn from_vec(items: Vec<Value>) -> Self {
        PVec::Array(Arc::new(items))
    }

    pub fn len(&self) -> usize {
        match self {
            PVec::Array(v) => v.len(),
            PVec::Tree(t) => t.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        match self {
            PVec::Array(v) => v.get(index),
            PVec::Tree(t) => t.get(index),
        }
    }

    pub fn first(&self) -> Option<&Value> {
        self.get(0)
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &Value> + '_> {
        match self {
            PVec::Array(v) => Box::new(v.iter()),
            PVec::Tree(t) => Box::new(t.iter()),
        }
    }

    /// Unique receiver: grow the Array in place (even past the threshold).
    /// Shared Array: `make_mut` copies, still Array.
    pub fn push_back_mut(&mut self, v: Value) {
        match self {
            PVec::Array(items) => {
                Arc::make_mut(items).push(v);
            }
            PVec::Tree(t) => {
                t.push_back_mut(v);
            }
        }
    }

    /// Persistent append. Array at/above the threshold promotes so further
    /// conj is RRB, not O(n) copy.
    pub fn push_back(&self, v: Value) -> Self {
        match self {
            PVec::Array(items) if items.len() >= PROMOTION_THRESHOLD => {
                let mut t = tree_from_slice(items);
                t.push_back_mut(v);
                PVec::Tree(t)
            }
            PVec::Array(items) => {
                let mut next = (**items).clone();
                next.push(v);
                PVec::Array(Arc::new(next))
            }
            PVec::Tree(t) => PVec::Tree(t.push_back(v)),
        }
    }

    /// Test-only: did persistent conj promote?
    pub fn is_tree(&self) -> bool {
        matches!(self, PVec::Tree(..))
    }
}

fn tree_from_slice(items: &[Value]) -> rpds::VectorSync<Value> {
    let mut t = rpds::VectorSync::new_sync();
    for x in items {
        t.push_back_mut(x.clone());
    }
    t
}

impl PartialEq for PVec {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for PVec {}

impl Hash for PVec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for elem in self.iter() {
            elem.hash(state);
        }
    }
}

impl FromIterator<Value> for PVec {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        PVec::from_vec(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn k(n: i64) -> Value {
        Value::i64(n)
    }
    fn hash_of(v: &PVec) -> u64 {
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        h.finish()
    }
    fn tree_of(n: i64) -> PVec {
        let mut t = rpds::VectorSync::new_sync();
        for i in 0..n {
            t.push_back_mut(k(i));
        }
        PVec::Tree(t)
    }

    #[test]
    fn from_vec_is_array_and_equals_tree_of_same_elements() {
        for n in [0i64, 1, 7, 8, 9, 64, 400] {
            let a = PVec::from_vec((0..n).map(k).collect());
            assert!(!a.is_tree(), "from_vec must stay Array at n={n}");
            let t = tree_of(n);
            assert!(t.is_tree() || n == 0, "setup tree at n={n}");
            assert_eq!(a, t, "Array != Tree at n={n} — representation leaked");
            assert_eq!(hash_of(&a), hash_of(&t), "hash differs by arm at n={n}");
        }
    }

    #[test]
    fn persistent_conj_past_threshold_promotes() {
        // Persistent `push_back` (`&self`), not `_mut`. Bind the result to a
        // new name — `v = v.push_back` is the no-rpds-rebuild-loop shape.
        let mut v = PVec::new();
        for i in 0..PROMOTION_THRESHOLD {
            let next = v.push_back(k(i as i64));
            assert!(!next.is_tree(), "promoted early at {i}");
            v = next;
        }
        let promoted = v.push_back(k(PROMOTION_THRESHOLD as i64));
        assert!(
            promoted.is_tree(),
            "never promoted after {} conj",
            PROMOTION_THRESHOLD + 1
        );
        let once = PVec::from_vec((0..=PROMOTION_THRESHOLD as i64).map(k).collect());
        assert_eq!(promoted, once);
        assert_eq!(hash_of(&promoted), hash_of(&once));
    }

    #[test]
    fn unique_push_back_mut_stays_array_past_threshold() {
        let mut v = PVec::new();
        for i in 0..64 {
            v.push_back_mut(k(i));
        }
        assert!(!v.is_tree(), "unique mut build must stay Array");
        assert_eq!(v.len(), 64);
    }

    #[test]
    fn a_vector_used_as_a_map_key_is_found_across_arms() {
        let array_key = PVec::from_vec((0..4i64).map(k).collect());
        let tree_key = tree_of(4);
        assert_eq!(array_key, tree_key);
        let found = Value::String(Arc::new("found".to_string()));
        let outer = crate::value::pmap::PMap::from_pairs([(
            Value::wat__core__PersistentVector(array_key.clone()),
            found.clone(),
        )]);
        assert_eq!(
            outer.get(&Value::wat__core__PersistentVector(tree_key)),
            Some(&found),
            "a vector-as-key entry must be found from the other arm"
        );
    }
}
