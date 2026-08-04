;; Arc 278 #57 round 1b — sanity probe for the eight newly-minted rows: the parametric
;; PersistentVector trio (Alias, type_params ["T"]) and the five Redispatch-class higher-order
;; combinators. Not a test file itself (see tests/rete for the durable gate); this is a
;; loadable, type-checked reference proving the new spellings resolve and, for the PV trio,
;; that a real parametric TypeScheme is now attached (round 1a's rows were all monomorphic).
(def :probe-pv-length (:wat::rete::PersistentVector/length (:wat::core::PersistentVector 1 2 3)))
(def :probe-pv-contains (:wat::rete::PersistentVector/contains? (:wat::core::PersistentVector 1 2 3) 2))
(def :probe-pv-get (:wat::rete::PersistentVector/get (:wat::core::PersistentVector 1 2 3) 1))

(def :probe-foldl
  (:wat::rete::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))

(def :probe-foldr
  (:wat::rete::foldr
    (:wat::core::fn [x <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x acc))
    0
    (:wat::core::PersistentVector 1 2 3)))

(def :probe-map
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::rete::map
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
      (:wat::core::PersistentVector 1 2 3))))

(def :probe-filter
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::rete::filter
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))
      (:wat::core::PersistentVector 1 2 3))))

;; `reduce` is a wat-level `defclause` (`wat/seq.wat`), not a checker special form like its
;; four siblings above — its rete row re-dispatches by AST head-substitution into the SAME
;; defclause-dispatch machinery a core-spelled call reaches (see check.rs's infer_rete_form,
;; the `:wat::core::reduce` arm).
(def :probe-reduce
  (:wat::rete::reduce
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))
