;; STONE 255.68 fixture — an untyped `(:wat::core::PersistentVector)` argument passed where the
;; declared parameter is `(wat.type/PersistentVector :- [wat.type/i64])`. Under
;; WAT_CHECK_TYPES=1 the checker's own inference records `i64` as the argument's element type —
;; from the checker, not from the text (the argument itself spells no element type at all).
(:wat::core::def :user::takes-vec
  (:wat::core::fn [v <- (wat.type/PersistentVector :- [wat.type/i64])] -> wat.type/nil
    nil))

(:wat::core::def :user::main
  (:wat::core::fn [] -> wat.type/nil
    (:user::takes-vec (wat.type/PersistentVector :- [wat.type/i64]))))
