;; tests/reflection/probe_arc255_spec_complete_yields_witness.wat
;; Fixture for yields_witness_applies_fn_to_42.
;; yields-witness applies f(42); f = fn [x <- i64] -> i64 (+ x 1) => f(42) = 43.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::intrinsic::yields-witness
    (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ x 1))))
