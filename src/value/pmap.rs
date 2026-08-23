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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::value::Value;

/// Rust-only identity for a map *instance*, copied on clone and minted on every
/// structural rewrite. Not part of `Eq` / `Hash` — two maps with the same
/// entries stay equal across arms and across intern ids.
fn next_intern() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Entries at or below this live in the array arm; `assoc` past it promotes to the trie.
/// Clojure's `PersistentArrayMap` boundary — chosen because it is a well-worn number from
/// elsewhere, not one fitted to this repo's own code.
pub const PROMOTION_THRESHOLD: usize = 8;

/// A persistent map that picks its representation from its own size.
#[derive(Debug, Clone)]
pub enum PMap {
    /// `<= PROMOTION_THRESHOLD` entries, insertion-ordered, linear scan. No HAMT allocation.
    /// Slice, not `Vec`: one-entry harvest is one alloc (`DESIGN-STONE-harvest-wrap-split`).
    /// The `u64` is a rust intern: clone-stable, ignored by `Eq`/`Hash`.
    Array(Arc<[(Value, Value)]>, u64),
    /// Above the threshold — the prior representation, unchanged.
    /// The `u64` is a rust intern: clone-stable, ignored by `Eq`/`Hash`.
    Trie(rpds::HashTrieMapSync<Value, Value>, u64),
}

impl Default for PMap {
    fn default() -> Self {
        PMap::new()
    }
}

impl PMap {
    pub fn new() -> Self {
        PMap::Array(Arc::from([]), next_intern())
    }

    /// Clone-stable rust identity. Copied on `clone`; minted on every
    /// structural rewrite (`assoc` / `dissoc` / `extend` that change
    /// entries). Not part of value equality.
    pub fn rust_identity(&self) -> u64 {
        match self {
            PMap::Array(_, id) | PMap::Trie(_, id) => *id,
        }
    }

    /// Build from an iterator, choosing the arm from the FINAL size — so a map built in one shot
    /// and the same map built by successive `assoc`s land in the same arm. Later duplicate keys
    /// win, matching `assoc`.
    ///
    /// Zero and one pair skip the growable accumulator: class-scan harvest
    /// is 40k one-entry maps (`DESIGN-STONE-one-entry-pmap-harvest`).
    pub fn from_pairs<I: IntoIterator<Item = (Value, Value)>>(pairs: I) -> Self {
        let mut iter = pairs.into_iter();
        let Some(first) = iter.next() else {
            return PMap::new();
        };
        let Some(second) = iter.next() else {
            return PMap::Array(Arc::from([first]), next_intern());
        };
        let mut acc: Vec<(Value, Value)> = Vec::new();
        acc.push(first);
        for (k, v) in std::iter::once(second).chain(iter) {
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
            PMap::Trie(t, next_intern())
        } else {
            PMap::Array(Arc::from(acc), next_intern())
        }
    }

    pub fn len(&self) -> usize {
        match self {
            PMap::Array(v, _) => v.len(),
            PMap::Trie(t, _) => t.size(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, k: &Value) -> Option<&Value> {
        match self {
            PMap::Array(v, _) => v.iter().find(|(ek, _)| ek == k).map(|(_, ev)| ev),
            PMap::Trie(t, _) => t.get(k),
        }
    }

    pub fn contains_key(&self, k: &Value) -> bool {
        self.get(k).is_some()
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (&Value, &Value)> + '_> {
        match self {
            PMap::Array(v, _) => Box::new(v.iter().map(|(k, val)| (k, val))),
            PMap::Trie(t, _) => Box::new(t.iter()),
        }
    }

    /// Insert, promoting when a NEW key would take the array past the threshold. Replacing an
    /// existing key never promotes — the size does not change, so the arm should not either.
    pub fn assoc(&self, k: Value, v: Value) -> Self {
        match self {
            PMap::Array(entries, _) => {
                if let Some(i) = entries.iter().position(|(ek, _)| *ek == k) {
                    let mut next = entries.to_vec();
                    next[i].1 = v;
                    return PMap::Array(Arc::from(next), next_intern());
                }
                if entries.len() + 1 > PROMOTION_THRESHOLD {
                    let mut t = rpds::HashTrieMapSync::new_sync();
                    for (ek, ev) in entries.iter() {
                        t.insert_mut(ek.clone(), ev.clone());
                    }
                    t.insert_mut(k, v);
                    return PMap::Trie(t, next_intern());
                }
                let mut next = entries.to_vec();
                next.push((k, v));
                PMap::Array(Arc::from(next), next_intern())
            }
            PMap::Trie(t, _) => {
                let mut next = t.clone();
                next.insert_mut(k, v);
                PMap::Trie(next, next_intern())
            }
        }
    }

    /// Apply many entries in ONE clone of the backing storage. `assoc` in a loop copies the whole
    /// Vec per key; this copies it once. Exists because the production caller (`extend_token`)
    /// folds an element's entire binding array into a token in a single act — the shape the trie
    /// arm already had (clone once, then `insert_mut` per key) and the array arm did not.
    ///
    /// Observationally identical to folding `assoc` over `pairs` — same entries, same
    /// later-key-wins, and the SAME arm (the array arm picks its arm from the FINAL length, just
    /// as `from_pairs` does, so a batch that crosses the threshold promotes exactly as successive
    /// `assoc` would). Zero pairs never clones the backing storage — the working copy is
    /// materialised lazily, on the first item.
    pub fn extend<I: IntoIterator<Item = (Value, Value)>>(&self, pairs: I) -> Self {
        match self {
            PMap::Array(entries, _) => {
                let mut next: Option<Vec<(Value, Value)>> = None;
                for (k, v) in pairs {
                    let vec = next.get_or_insert_with(|| entries.to_vec());
                    match vec.iter_mut().find(|(ek, _)| *ek == k) {
                        Some(slot) => slot.1 = v,
                        None => vec.push((k, v)),
                    }
                }
                match next {
                    None => self.clone(),
                    Some(vec) => {
                        if vec.len() > PROMOTION_THRESHOLD {
                            let mut t = rpds::HashTrieMapSync::new_sync();
                            for (k, v) in vec {
                                t.insert_mut(k, v);
                            }
                            PMap::Trie(t, next_intern())
                        } else {
                            PMap::Array(Arc::from(vec), next_intern())
                        }
                    }
                }
            }
            PMap::Trie(t, _) => {
                let mut next: Option<rpds::HashTrieMapSync<Value, Value>> = None;
                for (k, v) in pairs {
                    let m = next.get_or_insert_with(|| t.clone());
                    m.insert_mut(k, v);
                }
                match next {
                    None => self.clone(),
                    Some(m) => PMap::Trie(m, next_intern()),
                }
            }
        }
    }

    /// Remove. Never demotes — see the module doc: representation follows the high-water mark, so
    /// assoc/dissoc at the boundary cannot thrash the representation.
    pub fn dissoc(&self, k: &Value) -> Self {
        match self {
            PMap::Array(entries, _) => match entries.iter().position(|(ek, _)| ek == k) {
                None => self.clone(),
                Some(i) => {
                    let mut next = entries.to_vec();
                    next.remove(i);
                    PMap::Array(Arc::from(next), next_intern())
                }
            },
            PMap::Trie(t, _) => {
                let mut next = t.clone();
                next.remove_mut(k);
                PMap::Trie(next, next_intern())
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
        matches!(self, PMap::Trie(..))
    }

    /// Adopt an existing trie, CHOOSING THE ARM BY SIZE. Never wrap a trie directly as
    /// `PMap::Trie(t)` at a call site — a small map arriving that way would keep its HAMT
    /// forever and silently opt out of promotion, the exact thing this type exists to prevent.
    pub fn from_trie(t: rpds::HashTrieMapSync<Value, Value>) -> Self {
        if t.size() <= PROMOTION_THRESHOLD {
            PMap::Array(
                Arc::from(
                    t.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<Vec<_>>(),
                ),
                next_intern(),
            )
        } else {
            PMap::Trie(t, next_intern())
        }
    }

    /// The trie view, materialising one from the array arm when a reader genuinely needs rpds
    /// (e.g. a boundary to code that is intentionally still trie-only, such as `Token.bindings`).
    pub fn to_trie(&self) -> rpds::HashTrieMapSync<Value, Value> {
        match self {
            PMap::Trie(t, _) => t.clone(),
            PMap::Array(entries, _) => {
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
        let array = PMap::Array(Arc::from(pairs.clone()), next_intern());
        let mut t = rpds::HashTrieMapSync::new_sync();
        for (kk, vv) in pairs {
            t.insert_mut(kk, vv);
        }
        (array, PMap::Trie(t, next_intern()))
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
        let s = crate::edn_shim::value_to_edn_string_with(&outer_a_value, None);
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

    #[test]
    fn one_pair_from_pairs_is_the_array_arm_and_equals_assoc() {
        let once = PMap::from_pairs([(k(1), k(2))]);
        assert!(!once.is_trie());
        assert_eq!(once.len(), 1);
        let via_assoc = PMap::new().assoc(k(1), k(2));
        assert_eq!(once, via_assoc);
        assert_eq!(hash_of(&once), hash_of(&via_assoc));
        let empty = PMap::from_pairs(std::iter::empty::<(Value, Value)>());
        assert_eq!(empty, PMap::new());
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

    /// ★ THE EXTEND LAW — the wall Part A's RED gate proves: `m.extend(pairs)` must be
    /// observationally identical to folding `assoc` over the same `pairs` — same entries, same
    /// later-key-wins, AND the same arm (`is_trie()` agrees). Checked over sequences that stay
    /// under the threshold, land exactly on it, cross it, start from an already-`Trie` map, carry
    /// duplicate keys within one batch, and the empty batch — an arm check alone would let a
    /// silent representation drift through, so both entries and `is_trie()` are asserted every
    /// time.
    #[test]
    fn extend_matches_folded_assoc_same_entries_same_arm() {
        fn check(label: &str, start: PMap, pairs: Vec<(Value, Value)>) {
            let via_extend = start.extend(pairs.clone());
            let via_fold = pairs.into_iter().fold(start, |acc, (k, v)| acc.assoc(k, v));
            assert!(
                via_extend == via_fold,
                "{label}: extend and folded-assoc disagree on entries"
            );
            assert_eq!(
                via_extend.is_trie(),
                via_fold.is_trie(),
                "{label}: extend landed in a different ARM than folded-assoc (extend.is_trie()={}, fold.is_trie()={})",
                via_extend.is_trie(),
                via_fold.is_trie(),
            );
        }

        // (i) stays under the threshold: 0 entries + 3 new keys.
        check(
            "under threshold",
            PMap::new(),
            (0..3i64).map(|i| (k(i), k(i * 10))).collect(),
        );

        // (ii) lands exactly on PROMOTION_THRESHOLD (8): 0 + 8 new keys.
        check(
            "lands exactly on 8",
            PMap::new(),
            (0..PROMOTION_THRESHOLD as i64).map(|i| (k(i), k(i * 10))).collect(),
        );

        // (iii) crosses the threshold: 3 existing + 6 new keys = 9.
        check(
            "crosses the threshold",
            PMap::from_pairs((0..3i64).map(|i| (k(i), k(i)))),
            (3..9i64).map(|i| (k(i), k(i * 10))).collect(),
        );

        // (iv) a batch applied to a map that is ALREADY a Trie.
        let already_trie = PMap::from_pairs((0..12i64).map(|i| (k(i), k(i))));
        assert!(already_trie.is_trie(), "setup: expected the trie arm");
        check(
            "already a trie",
            already_trie,
            (12..15i64).map(|i| (k(i), k(i * 10))).collect(),
        );

        // (v) duplicate keys within one batch — later value must win, both directly (via extend)
        // and through the fold (which applies them in order too).
        check(
            "duplicate keys within one batch, under threshold",
            PMap::from_pairs((0..2i64).map(|i| (k(i), k(i)))),
            vec![(k(5), k(500)), (k(5), k(501)), (k(0), k(999))],
        );
        let dup_result = PMap::from_pairs((0..2i64).map(|i| (k(i), k(i))))
            .extend(vec![(k(5), k(500)), (k(5), k(501)), (k(0), k(999))]);
        assert_eq!(dup_result.get(&k(5)), Some(&k(501)), "later duplicate key must win");
        assert_eq!(dup_result.get(&k(0)), Some(&k(999)), "later duplicate key must win (existing key)");

        // duplicate keys that cross the threshold — the FINAL de-duplicated length decides the
        // arm, not the raw pair count.
        check(
            "duplicate keys within one batch, crossing the threshold",
            PMap::new(),
            vec![
                (k(0), k(0)), (k(1), k(1)), (k(2), k(2)), (k(3), k(3)),
                (k(4), k(4)), (k(5), k(5)), (k(6), k(6)), (k(7), k(7)),
                (k(8), k(8)), (k(8), k(80)), // 9 distinct keys, one repeated -> still crosses
            ],
        );

        // (vi) the empty batch — both arms, and must not clone the backing Vec (asserted via
        // pointer identity on the Array arm's Arc).
        let empty_array = PMap::from_pairs((0..3i64).map(|i| (k(i), k(i))));
        assert!(!empty_array.is_trie(), "setup: expected the array arm");
        let array_ptr_before = match &empty_array {
            PMap::Array(v, _) => Arc::as_ptr(v),
            PMap::Trie(..) => panic!("setup: expected the array arm"),
        };
        let extended_empty_array = empty_array.extend(Vec::<(Value, Value)>::new());
        assert!(extended_empty_array == empty_array, "empty extend must not change entries");
        assert!(!extended_empty_array.is_trie(), "empty extend must not change the arm");
        match &extended_empty_array {
            PMap::Array(v, _) => assert_eq!(
                Arc::as_ptr(v),
                array_ptr_before,
                "empty extend must not clone the backing Vec (Arc pointer changed)"
            ),
            PMap::Trie(..) => panic!("empty extend on an Array must not promote"),
        }

        let empty_trie = PMap::from_pairs((0..12i64).map(|i| (k(i), k(i))));
        assert!(empty_trie.is_trie(), "setup: expected the trie arm");
        check("empty batch on a trie", empty_trie, Vec::new());
    }

    /// Item 12 — clone shares intern; a structural rewrite mints a new one.
    /// `insert` overlays facts by cloning the network PMap; fire looks the
    /// arm up by this id. If clone minted, every overlay would rebuild.
    #[test]
    fn rust_identity_survives_clone_and_empty_extend_not_rewrite() {
        let array = PMap::from_pairs((0..3i64).map(|i| (k(i), k(i))));
        let trie = PMap::from_pairs((0..12i64).map(|i| (k(i), k(i))));
        assert!(!array.is_trie());
        assert!(trie.is_trie());
        assert_eq!(array.rust_identity(), array.clone().rust_identity());
        assert_eq!(trie.rust_identity(), trie.clone().rust_identity());
        assert_eq!(
            array.rust_identity(),
            array.extend(Vec::<(Value, Value)>::new()).rust_identity(),
            "empty extend must keep intern (same as clone)"
        );
        assert_eq!(
            trie.rust_identity(),
            trie.extend(Vec::<(Value, Value)>::new()).rust_identity(),
            "empty extend must keep intern (same as clone)"
        );
        assert_ne!(
            array.rust_identity(),
            array.assoc(k(99), k(1)).rust_identity(),
            "assoc must mint — the network changed"
        );
        assert_ne!(
            trie.rust_identity(),
            trie.assoc(k(99), k(1)).rust_identity(),
            "assoc must mint — the network changed"
        );
    }
}
