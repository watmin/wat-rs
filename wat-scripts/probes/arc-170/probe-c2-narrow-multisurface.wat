;; NARROW-2: a plain record satisfying a MONOMORPHIC surface (Flat) AND a PARAMETRIC one (Pair2 :- [A B])
;; — exactly what a service Handle does (it satisfies Capability mono, + we add Dialable parametric).
;; parametric receiver errors → bug is multi-surface (pre-existing mono satisfaction blocks parametric)
;; clean → bug is specific to the AUTO-EMITTED service Handle
(:wat::core::defsurface :probe::Flat :nature :wat::core::Struct
  :features [(tag [self <- :probe::Flat] -> :wat::core::String)])
(:wat::core::defsurface :probe::Pair2 :- [A B] :nature :wat::core::Struct
  :features [(fst [self <- (:probe::Pair2 :- [A B])] -> :A)])
(:wat::core::defrecord :probe::Multi [i <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::extend-type :probe::Multi :probe::Flat
  (tag [self] (:probe::Multi/s self)))
(:wat::core::extend-type :probe::Multi (:probe::Pair2 :- [:wat::core::i64 :wat::core::String])
  (fst [self] (:probe::Multi/i self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m  (:probe::Multi :i 42 :s "hi")
     ok (:wat::core::ann-form (:probe::Pair2/fst m) :wat::core::i64)]
    (:wat::kernel::println "narrowed2")))
