;; NARROW-2: a plain record satisfying a MONOMORPHIC surface (Flat) AND a PARAMETRIC one (Pair2 :- [A B])
;; — exactly what a service Handle does (it satisfies Capability mono, + we add Dialable parametric).
;; parametric receiver errors → bug is multi-surface (pre-existing mono satisfaction blocks parametric)
;; clean → bug is specific to the AUTO-EMITTED service Handle
;; CLAIM: (Pair2/fst m) == 42 — a record satisfying BOTH a mono and a parametric surface still
;; dispatches the parametric one correctly.
(:wat::core::defsurface :probe::Flat :nature wat.type/Struct
  :features [(tag [self <- :probe::Flat] -> wat.type/String)])
(:wat::core::defsurface :probe::Pair2 :- [A B] :nature wat.type/Struct
  :features [(fst [self <- (:probe::Pair2 :- [A B])] -> :A)])
(:wat::core::defrecord :probe::Multi [i <- wat.type/i64  s <- wat.type/String])
(:wat::core::extend-type :probe::Multi :probe::Flat
  (tag [self] (:probe::Multi/s self)))
(:wat::core::extend-type :probe::Multi (:probe::Pair2 :- [wat.type/i64 wat.type/String])
  (fst [self] (:probe::Multi/i self)))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [m  (:probe::Multi :i 42 :s "hi")
     ok (:wat::core::ann-form (:probe::Pair2/fst m) :wat::core::i64)]
    (:wat::core::do
      (:wat::test::assert-eq ok 42)
      (:wat::kernel::println "narrowed2"))))
