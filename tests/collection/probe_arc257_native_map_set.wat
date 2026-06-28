;; tests/collection/probe_arc257_native_map_set.wat — co-located fixture.
;; Arc 257 (EDN-native collections), Slice 257.1.
;; Verifies {k v} map literals and #{x y z} set literals evaluate correctly.

;; Probe 1: single-entry map literal → i64 length 1
(:wat::core::defn :t::probe1-map-single [] -> :wat::core::i64
  (:wat::core::let
    [m {:a 42}]
    (:wat::core::length m)))

;; Probe 2: multi-entry map literal → i64 length 2
(:wat::core::defn :t::probe2-map-multi [] -> :wat::core::i64
  (:wat::core::let
    [m {:x 10 :y 20}]
    (:wat::core::length m)))

;; Probe 3: set literal with contains? → bool true
(:wat::core::defn :t::probe3-set-contains [] -> :wat::core::bool
  (:wat::core::let
    [s #{1 2 3}]
    (:wat::core::contains? s 2)))
