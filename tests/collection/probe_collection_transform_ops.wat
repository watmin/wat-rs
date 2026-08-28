;; tests/collection/probe_collection_transform_ops.wat — co-located fixture.
;; Perimeter-closure probes for the collection dispatch home.

;; item1a: length of non-empty (List :- [i64])(10,20,30)
(:wat::core::defn :t::item1a-list-len-nonempty [] -> :wat::core::i64
  (:wat::core::length (:wat::core::List 10 20 30)))

;; item1b: length of empty List
(:wat::core::defn :t::item1b-list-len-empty [] -> :wat::core::i64
  (:wat::core::length (:wat::core::List)))

;; item1c: empty? on non-empty List
(:wat::core::defn :t::item1c-list-empty-nonempty [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::List 1 2 3)))

;; item1d: empty? on empty List
(:wat::core::defn :t::item1d-list-empty-empty [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::List)))

;; item4a: zip happy path — length of zipped result
(:wat::core::defn :t::item4a-zip-happy-len [] -> :wat::core::i64
  (:wat::core::let
    [zipped (:wat::seq::zip
               (:wat::core::Vector :wat::core::i64 1 2 3)
               (:wat::core::Vector :wat::core::i64 4 5 6))]
    (:wat::vec::length zipped)))

;; item4b: zip with empty first vector → length 0
(:wat::core::defn :t::item4b-zip-empty-len [] -> :wat::core::i64
  (:wat::core::let
    [zipped (:wat::seq::zip
               (:wat::core::Vector :wat::core::i64)
               (:wat::core::Vector :wat::core::i64 1 2 3))]
    (:wat::vec::length zipped)))

;; item4c: window happy path — 3 windows of size 2 over 4 elements
(:wat::core::defn :t::item4c-window-happy-len [] -> :wat::core::i64
  (:wat::vec::length
    (:wat::seq::window
       (:wat::core::Vector :wat::core::i64 1 2 3 4)
       2)))

;; item4d: window n > len → empty output
(:wat::core::defn :t::item4d-window-n-gt-len [] -> :wat::core::i64
  (:wat::vec::length
    (:wat::seq::window
       (:wat::core::Vector :wat::core::i64 1 2)
       5)))

;; item4e: remove-at happy path — length after removal
(:wat::core::defn :t::item4e-remove-at-happy-len [] -> :wat::core::i64
  (:wat::vec::length
    (:wat::seq::remove-at
       (:wat::core::Vector :wat::core::i64 10 20 30)
       1)))

;; item4f: remove-at out-of-range — length unchanged
(:wat::core::defn :t::item4f-remove-at-oob-len [] -> :wat::core::i64
  (:wat::vec::length
    (:wat::seq::remove-at
       (:wat::core::Vector :wat::core::i64 10 20 30)
       99)))

;; item4g: map-indexed happy — sum of indices (Arc 255 Stone HOME-9: the deleted
;; `:wat::std::list::map-with-index` is replaced by `:wat::core::map-indexed`, NOT a drop-in —
;; arg order flips (coll,fn)->(fn,coll), closure params flip (item,i)->(i,item), and the result
;; is a lazy Stream, not an eager Vector. `foldl` already accepts a Stream directly (Seqable-
;; generic), so no `into` is needed here — only item4h's `:wat::vec::length` call needs one.
(:wat::core::defn :t::item4g-map-indexed-happy [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc x))
    0
    (:wat::core::map-indexed
      (:wat::core::fn [i <- :wat::core::i64 _v <- :wat::core::i64] -> :wat::core::i64 i)
      (:wat::core::Vector :wat::core::i64 10 20 30))))

;; item4h: map-indexed empty input → length 0. `:wat::vec::length` is Vector-only, so `into []`
;; materializes the lazy Stream first.
(:wat::core::defn :t::item4h-map-indexed-empty [] -> :wat::core::i64
  (:wat::vec::length
    (:wat::core::into []
      (:wat::core::map-indexed
        (:wat::core::fn [i <- :wat::core::i64 v <- :wat::core::i64] -> :wat::core::i64 i)
        (:wat::core::Vector :wat::core::i64)))))

;; item4i: find-last-index happy — returns index of last x > 10
(:wat::core::defn :t::item4i-find-last-idx-happy [] -> :wat::core::i64
  (:wat::core::match
    (:wat::core::find-last-index
      (:wat::core::Vector :wat::core::i64 5 12 3 18 7)
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
        (:wat::i64::> x 10)))
    
    ((:wat::core::Some i) i)
    (:wat::core::None -1)))

;; item4j: find-last-index no match → None (sentinel -1)
(:wat::core::defn :t::item4j-find-last-idx-none [] -> :wat::core::i64
  (:wat::core::match
    (:wat::core::find-last-index
      (:wat::core::Vector :wat::core::i64 1 2 3)
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
        (:wat::i64::> x 99)))
    
    ((:wat::core::Some i) i)
    (:wat::core::None -1)))

;; item5a: conj must not mutate v0 — v0 length stays 2
(:wat::core::defn :t::item5a-conj-immutable-len [] -> :wat::core::i64
  (:wat::core::let
    [v0 (:wat::core::Vector :wat::core::i64 1 2)
     _  (:wat::vec::conj v0 3)]
    (:wat::vec::length v0)))

;; item5b: conj returns new vector of length 3
(:wat::core::defn :t::item5b-conj-new-len [] -> :wat::core::i64
  (:wat::core::let
    [v0 (:wat::core::Vector :wat::core::i64 1 2)
     v1 (:wat::vec::conj v0 3)]
    (:wat::vec::length v1)))

;; item5c: conj appends element at last position
(:wat::core::defn :t::item5c-conj-new-elem [] -> :wat::core::i64
  (:wat::core::let
    [v0 (:wat::core::Vector :wat::core::i64 1 2)
     v1 (:wat::vec::conj v0 99)]
    (:wat::core::match
      (:wat::vec::get v1 2)
      
      ((:wat::core::Some x) x)
      (:wat::core::None -1))))
