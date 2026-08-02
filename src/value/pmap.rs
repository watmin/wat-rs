//! `PMap` — the promoting map: an array below the threshold, a trie above it.
//!
//! `:wat::core::PersistentMap` was an unconditional `rpds::HashTrieMapSync`, so every `{:a 1}` any
//! wat program writes allocated a HAMT. One representation chosen globally is a claim about the
//! size of maps users write; promoting per instance makes no claim at all — each map picks from its
//! own size, at runtime. Threshold 8, Clojure's `PersistentArrayMap` boundary.
//!
//! ## The wall — representation must be UNOBSERVABLE
//!
//! Promotion is **one-way**: `assoc` past the threshold promotes; `dissoc` below it does not
//! demote, so a map's representation is a function of its high-water mark rather than its whole
//! history. The consequence is the invariant everything else here exists to hold: **two maps with
//! the same entries are the same value, whichever arm holds them.** `PartialEq` compares entry
//! sets, never containers; `Hash` runs one routine over both arms. Get this wrong and a map used as
//! a key silently misses — the one failure mode here that is not loud.
//!
//! Iteration order is deliberately NOT part of the contract (the trie has no meaningful order, so
//! promising one would be a lie the array arm could keep and the trie arm could not).

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::value::Value;

/// Entries at or below this live in the array arm; `assoc` past it promotes to the trie.
/// Clojure's `PersistentArrayMap` boundary — chosen because it is a well-worn number from
/// elsewhere, not one fitted to this repo's own code.
pub const PROMOTION_THRESHOLD: usize = 8;

/// A persistent map that picks its representation from its own size.
#[derive(Debug, Clone)]
pub enum PMap {
    /// `<= PROMOTION_THRESHOLD` entries, insertion-ordered, linear scan. No HAMT allocation.
    Array(Arc<Vec<(Value, Value)>>),
    /// Above the threshold — the prior representation, unchanged.
    Trie(rpds::HashTrieMapSync<Value, Value>),
}

impl Default for PMap {
    fn default() -> Self {
        PMap::new()
    }
}

impl PMap {
    pub fn new() -> Self {
        PMap::Array(Arc::new(Vec::new()))
    }

    /// Build from an iterator, choosing the arm from the FINAL size — so a map built in one shot
    /// and the same map built by successive `assoc`s land in the same arm. Later duplicate keys
    /// win, matching `assoc`.
    pub fn from_pairs<I: IntoIterator<Item = (Value, Value)>>(pairs: I) -> Self {
        let mut acc: Vec<(Value, Value)> = Vec::new();
        for (k, v) in pairs {
            match acc.iter_mut().find(|(ek, _)| *ek == k) {
                Some(slot) => slot.1 = v,
                None => acc.push((k, v)),
            }
        }
        if acc.len() > PROMOTION_THRESHOLD {
            let mut t = rpds::HashTrieMapSync::new_sync();
            for (k, v) in acc {
                t.insert_mut(k, v);
            }
            PMap::Trie(t)
        } else {
            PMap::Array(Arc::new(acc))
        }
    }

    pub fn len(&self) -> usize {
        match self {
            PMap::Array(v) => v.len(),
            PMap::Trie(t) => t.size(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, k: &Value) -> Option<&Value> {
        match self {
            PMap::Array(v) => v.iter().find(|(ek, _)| ek == k).map(|(_, ev)| ev),
            PMap::Trie(t) => t.get(k),
        }
    }

    pub fn contains_key(&self, k: &Value) -> bool {
        self.get(k).is_some()
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&Value, &Value)> + '_> {
        match self {
            PMap::Array(v) => Box::new(v.iter().map(|(k, val)| (k, val))),
            PMap::Trie(t) => Box::new(t.iter()),
        }
    }

    /// Insert, promoting when a NEW key would take the array past the threshold. Replacing an
    /// existing key never promotes — the size does not change, so the arm should not either.
    pub fn assoc(&self, k: Value, v: Value) -> Self {
        match self {
            PMap::Array(entries) => {
                if let Some(i) = entries.iter().position(|(ek, _)| *ek == k) {
                    let mut next = (**entries).clone();
                    next[i].1 = v;
                    return PMap::Array(Arc::new(next));
                }
                if entries.len() + 1 > PROMOTION_THRESHOLD {
                    let mut t = rpds::HashTrieMapSync::new_sync();
                    for (ek, ev) in entries.iter() {
                        t.insert_mut(ek.clone(), ev.clone());
                    }
                    t.insert_mut(k, v);
                    return PMap::Trie(t);
                }
                let mut next = (**entries).clone();
                next.push((k, v));
                PMap::Array(Arc::new(next))
            }
            PMap::Trie(t) => {
                let mut next = t.clone();
                next.insert_mut(k, v);
                PMap::Trie(next)
            }
        }
    }

    /// Remove. Never demotes — see the module doc: representation follows the high-water mark, so
    /// assoc/dissoc at the boundary cannot thrash the representation.
    pub fn dissoc(&self, k: &Value) -> Self {
        match self {
            PMap::Array(entries) => match entries.iter().position(|(ek, _)| ek == k) {
                None => self.clone(),
                Some(i) => {
                    let mut next = (**entries).clone();
                    next.remove(i);
                    PMap::Array(Arc::new(next))
                }
            },
            PMap::Trie(t) => {
                let mut next = t.clone();
                next.remove_mut(k);
                PMap::Trie(next)
            }
        }
    }

    pub fn keys(&self) -> Vec<Value> {
        self.iter().map(|(k, _)| k.clone()).collect()
    }

    pub fn values(&self) -> Vec<Value> {
        self.iter().map(|(_, v)| v.clone()).collect()
    }

    /// Which arm holds this map. Exposed ONLY so a test can prove promotion actually fired — a
    /// promotion test that never promotes is vacuous, and nothing else should branch on this.
    pub fn is_trie(&self) -> bool {
        matches!(self, PMap::Trie(_))
    }

    /// Adopt an existing trie, CHOOSING THE ARM BY SIZE. Never wrap a trie directly as
    /// `PMap::Trie(t)` at a call site — a small map arriving that way would keep its HAMT
    /// forever and silently opt out of promotion, the exact thing this type exists to prevent.
    pub fn from_trie(t: rpds::HashTrieMapSync<Value, Value>) -> Self {
        if t.size() <= PROMOTION_THRESHOLD {
            PMap::Array(Arc::new(t.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
        } else {
            PMap::Trie(t)
        }
    }

    /// The trie view, materialising one from the array arm when a reader genuinely needs rpds
    /// (e.g. a boundary to code that is intentionally still trie-only, such as `Token.bindings`).
    pub fn to_trie(&self) -> rpds::HashTrieMapSync<Value, Value> {
        match self {
            PMap::Trie(t) => t.clone(),
            PMap::Array(entries) => {
                let mut t = rpds::HashTrieMapSync::new_sync();
                for (k, v) in entries.iter() {
                    t.insert_mut(k.clone(), v.clone());
                }
                t
            }
        }
    }
}

/// Entry-SET equality, never container equality. This is the arm that made the stone: the previous
/// `(PersistentMap(a), PersistentMap(b)) => a == b` delegated to rpds' `PartialEq`, which is
/// representation-dependent and would have called an Array and a Trie holding identical entries
/// unequal.
impl PartialEq for PMap {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

impl Eq for PMap {}

/// The SAME order-independent routine over both arms — sort `(key_hash, value_hash)` pairs, hash
/// the sorted vector. Carried over verbatim from `impl Hash for Value`'s prior map arm
/// (arc-278-0a), which never touched the container, so it was already representation-agnostic
/// before anything needed it to be.
impl Hash for PMap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use std::collections::hash_map::DefaultHasher;
        let mut pair_hashes: Vec<(u64, u64)> = self
            .iter()
            .map(|(k, v)| {
                let mut kh = DefaultHasher::new();
                k.hash(&mut kh);
                let mut vh = DefaultHasher::new();
                v.hash(&mut vh);
                (kh.finish(), vh.finish())
            })
            .collect();
        pair_hashes.sort_unstable();
        pair_hashes.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn k(n: i64) -> Value {
        Value::i64(n)
    }
    fn hash_of(m: &PMap) -> u64 {
        let mut h = DefaultHasher::new();
        m.hash(&mut h);
        h.finish()
    }
    /// Same entries, forced into each arm, so every law below is checked ACROSS representations.
    fn both_arms(n: i64) -> (PMap, PMap) {
        let pairs: Vec<(Value, Value)> = (0..n).map(|i| (k(i), k(i * 10))).collect();
        let array = PMap::Array(Arc::new(pairs.clone()));
        let mut t = rpds::HashTrieMapSync::new_sync();
        for (kk, vv) in pairs {
            t.insert_mut(kk, vv);
        }
        (array, PMap::Trie(t))
    }

    /// ★ THE CROSS-REPRESENTATION LAW — the wall this stone stands on. A wrong answer here is
    /// SILENT: a map used as a key simply misses.
    #[test]
    fn same_entries_are_equal_and_hash_equal_in_either_arm() {
        for n in [0i64, 1, 7, 8, 9, 64] {
            let (a, t) = both_arms(n);
            assert_eq!(a.len(), t.len(), "n={n}");
            assert!(a == t, "Array != Trie at n={n} — representation leaked into equality");
            assert!(t == a, "Trie != Array at n={n} — equality is not symmetric");
            assert_eq!(hash_of(&a), hash_of(&t), "hash differs by representation at n={n}");
        }
    }

    /// ★ THE MAP-AS-KEY LAW — the row `DESIGN-STONE-promoting-map.md` names as the only SILENT
    /// failure mode in this stone. `{{:some :map} :as-a-key}` is legal EDN and round-trips today,
    /// so a peer can send one over the wire; if cross-arm key lookup breaks, the map just misses
    /// — no error, no panic, a wrong answer. Built one key via a small build (lands Array) and an
    /// equal key via build-past-the-threshold-then-`dissoc`-back (stays Trie — promotion is
    /// one-way), then looked up both directions, then through an EDN read -> write -> read round
    /// trip.
    #[test]
    fn a_map_used_as_a_key_is_found_across_arms() {
        // Same entries, one arm each.
        let array_key = PMap::from_pairs((0..4i64).map(|i| (k(i), k(i))));
        assert!(!array_key.is_trie(), "setup: expected the array arm");

        let mut trie_key = PMap::from_pairs((0..12i64).map(|i| (k(i), k(i))));
        assert!(trie_key.is_trie(), "setup: expected promotion to fire");
        for i in 4..12i64 {
            trie_key = trie_key.dissoc(&k(i));
        }
        assert!(trie_key.is_trie(), "setup: dissoc must not demote — see the contract above");
        assert!(array_key == trie_key, "setup: the two keys must be equal entry-sets");

        let found = Value::String(Arc::new("found".to_string()));

        // Stored under the ARRAY-arm key, looked up with the TRIE-arm key.
        let outer_a = PMap::from_pairs([(
            Value::wat__core__PersistentMap(array_key.clone()),
            found.clone(),
        )]);
        assert_eq!(
            outer_a.get(&Value::wat__core__PersistentMap(trie_key.clone())),
            Some(&found),
            "a Trie-arm key must find an entry stored under its Array-arm twin"
        );

        // The reverse — stored under the TRIE-arm key, looked up with the ARRAY-arm key.
        let outer_b = PMap::from_pairs([(
            Value::wat__core__PersistentMap(trie_key.clone()),
            found.clone(),
        )]);
        assert_eq!(
            outer_b.get(&Value::wat__core__PersistentMap(array_key.clone())),
            Some(&found),
            "an Array-arm key must find an entry stored under its Trie-arm twin"
        );

        // The same, through an EDN read -> write -> read round trip — the wire-visible case.
        let outer_a_value = Value::wat__core__PersistentMap(outer_a);
        let s = crate::edn_shim::value_to_edn_string(&outer_a_value);
        let back = crate::edn_shim::edn_string_to_value(&s).expect("round-trip parse");
        let back_pm = match back {
            Value::wat__core__PersistentMap(m) => m,
            other => panic!("must round-trip to a PersistentMap, got {other:?}"),
        };
        assert_eq!(
            back_pm.get(&Value::wat__core__PersistentMap(trie_key.clone())),
            Some(&found),
            "a map-as-key entry must survive an EDN round trip and still be found from the other arm"
        );
    }

    /// Promotion must actually FIRE, and the two build paths must converge. A promotion test where
    /// nothing promotes is vacuous.
    #[test]
    fn assoc_promotes_past_the_threshold_and_both_build_paths_agree() {
        let mut built = PMap::new();
        for i in 0..=(PROMOTION_THRESHOLD as i64) {
            assert!(!built.is_trie(), "promoted early at {i} entries");
            built = built.assoc(k(i), k(i * 10));
        }
        assert!(
            built.is_trie(),
            "never promoted after {} assocs — this test proves nothing",
            PROMOTION_THRESHOLD + 1
        );
        let at_once = PMap::from_pairs((0..=(PROMOTION_THRESHOLD as i64)).map(|i| (k(i), k(i * 10))));
        assert!(at_once.is_trie(), "one-shot build did not choose the trie arm");
        assert!(built == at_once, "successive assoc and one-shot build disagree");
        assert_eq!(hash_of(&built), hash_of(&at_once));
    }

    /// Replacing a key must not promote — the size does not change, so the arm must not either.
    #[test]
    fn replacing_an_existing_key_never_promotes() {
        let mut m = PMap::from_pairs((0..PROMOTION_THRESHOLD as i64).map(|i| (k(i), k(i))));
        assert!(!m.is_trie());
        for i in 0..PROMOTION_THRESHOLD as i64 {
            m = m.assoc(k(i), k(i * 100));
        }
        assert!(!m.is_trie(), "replacing existing keys promoted — size never grew");
        assert_eq!(m.len(), PROMOTION_THRESHOLD);
        assert_eq!(m.get(&k(3)), Some(&k(300)));
    }

    /// One-way: dissoc below the threshold keeps the trie arm, and stays EQUAL to the array-built
    /// twin — which is why the cross-representation law is load-bearing rather than cosmetic.
    #[test]
    fn dissoc_does_not_demote_but_stays_equal_to_its_array_twin() {
        let mut m = PMap::from_pairs((0..12i64).map(|i| (k(i), k(i))));
        assert!(m.is_trie());
        for i in 4..12i64 {
            m = m.dissoc(&k(i));
        }
        assert!(m.is_trie(), "demoted on dissoc — the contract is one-way promotion");
        assert_eq!(m.len(), 4);
        let twin = PMap::from_pairs((0..4i64).map(|i| (k(i), k(i))));
        assert!(!twin.is_trie());
        assert!(m == twin, "a dissoc'd trie != its array twin with the same entries");
        assert_eq!(hash_of(&m), hash_of(&twin));
    }

    /// Insertion order must not reach equality or hash — the trie has no order to promise.
    #[test]
    fn insertion_order_is_not_observable() {
        let fwd = PMap::from_pairs((0..5i64).map(|i| (k(i), k(i))));
        let rev = PMap::from_pairs((0..5i64).rev().map(|i| (k(i), k(i))));
        assert!(fwd == rev);
        assert_eq!(hash_of(&fwd), hash_of(&rev));
    }
}
