;; tests/collection/probe_map_container.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). All valid defns; startup type-checks all.

;; ── record preambles ──

(:wat::core::defrecord :probe::mr::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :probe::mr::Coord [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::holon::defrecord :probe::mr::Volt [value <- :wat::core::i64])
(:wat::core::defrecord :probe::rgal::Sensor [id <- :wat::core::i64  label <- :wat::core::String])
(:wat::core::defrecord :probe::rgal::Sensor2 [id <- :wat::core::i64])
(:wat::core::defrecord :probe::rgal::Node [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :probe::rgal::Node2 [x <- :wat::core::i64])
(:wat::core::defrecord :probe::rgal::Triple [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])
(:wat::core::defrecord :probe::rgal::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

;; ── assoc round-trip — HashMap ──

(:wat::core::defn :p::hashmap-assoc-key-present [] -> :wat::core::bool
  (:wat::core::let
    [m  (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m2 (:wat::core::assoc m "answer" 42)]
    (:wat::hashmap::contains-key? m2 "answer")))

(:wat::core::defn :p::hashmap-assoc-type-preserving [] -> :wat::core::bool
  (:wat::core::let
    [m  (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
     m2 (:wat::core::assoc m "k" 1)]
    (:wat::hashmap::contains-key? m2 "k")))

;; ── assoc round-trip — PersistentMap ──

(:wat::core::defn :p::persistentmap-assoc-length-grows [] -> :wat::core::i64
  (:wat::core::let
    [pm  (:wat::core::PersistentMap :a 1)
     pm2 (:wat::core::assoc pm :b 2)]
    (:wat::map::length pm2)))

(:wat::core::defn :p::persistentmap-assoc-immutable [] -> :wat::core::i64
  (:wat::core::let
    [pm  (:wat::core::PersistentMap :a 1)
     _   (:wat::core::assoc pm :b 2)]
    (:wat::map::length pm)))

;; ── assoc round-trip — base Record ──

(:wat::core::defn :p::base-record-assoc-field-updated [] -> :wat::core::i64
  (:wat::core::let
    [pt  (:probe::mr::Pt :x 3 :y 4)
     pt2 (:wat::core::assoc pt :y 99)]
    (:probe::mr::Pt/y pt2)))

(:wat::core::defn :p::base-record-assoc-preserves-other-fields [] -> :wat::core::i64
  (:wat::core::let
    [c  (:probe::mr::Coord :x 10 :y 20)
     c2 (:wat::core::assoc c :y 99)]
    (:probe::mr::Coord/x c2)))

;; ── assoc round-trip — holonic Record ──

(:wat::core::defn :p::holonic-record-assoc-field-updated [] -> :wat::core::i64
  (:wat::core::let
    [v  (:probe::mr::Volt :value 10)
     v2 (:wat::core::assoc v :value 77)]
    (:probe::mr::Volt/value v2)))

;; ── Record get ──

(:wat::core::defn :p::record-get-existing-field [] -> (:wat::core::Option :- [:wat::core::Value])
  (:wat::core::let
    [s (:probe::rgal::Sensor :id 42 :label "temp")]
    (:wat::core::get s :id)))

(:wat::core::defn :p::record-get-missing-field [] -> (:wat::core::Option :- [:wat::core::Value])
  (:wat::core::let
    [s (:probe::rgal::Sensor2 :id 7)]
    (:wat::core::get s :no-such-field)))

;; ── Record contains? ──

(:wat::core::defn :p::record-contains-existing [] -> :wat::core::bool
  (:wat::core::let
    [n (:probe::rgal::Node :x 1 :y 2)]
    (:wat::core::contains? n :x)))

(:wat::core::defn :p::record-contains-missing [] -> :wat::core::bool
  (:wat::core::let
    [n (:probe::rgal::Node2 :x 5)]
    (:wat::core::contains? n :z)))

;; ── Record length ──

(:wat::core::defn :p::record-length [] -> :wat::core::i64
  (:wat::core::let
    [t (:probe::rgal::Triple :a 1 :b 2 :c 3)]
    (:wat::core::length t)))

;; ── Record empty? ──

(:wat::core::defn :p::record-empty-nonempty [] -> :wat::core::bool
  (:wat::core::let
    [p (:probe::rgal::Pair :a 10 :b 20)]
    (:wat::core::empty? p)))
