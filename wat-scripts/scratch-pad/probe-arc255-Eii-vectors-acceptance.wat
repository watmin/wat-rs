;; arc 255 Stone E-ii — the acceptance probe: every verb of BOTH families, under the new
;; spellings, asserts a concrete result. 6 PersistentVector verbs + 7 Vector verbs = 13.
(:wat::core::defn :user::check [label <- :wat::core::String cond <- :wat::core::bool] -> :wat::core::nil
  (:wat::core::if cond
    (:wat::kernel::println (:wat::string::concat "ok   " label))
    (:wat::kernel::assertion-failed! (:wat::string::concat "FAIL " label) :wat::core::None :wat::core::None)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    ;; ── PersistentVector, 6 verbs: concat, conj, contains?, empty?, get, length ──────────
    [pv0     (:wat::core::PersistentVector 1 2 3)
     pv-e    (:wat::core::PersistentVector)
     pv1     (:wat::vector::conj pv0 4)
     pv2     (:wat::vector::concat pv0 (:wat::core::PersistentVector 4 5))]
    (:wat::core::do
      (:user::check "vector::length"    (:wat::core::= (:wat::vector::length pv0) 3))
      (:user::check "vector::empty?/t"  (:wat::vector::empty? pv-e))
      (:user::check "vector::empty?/f"  (:wat::core::not (:wat::vector::empty? pv0)))
      (:user::check "vector::contains?" (:wat::vector::contains? pv0 2))
      (:user::check "vector::get/some"  (:wat::core::match (:wat::vector::get pv0 0)
                                           ((:wat::core::Some x) (:wat::core::= x 1))
                                           (:wat::core::None false)))
      (:user::check "vector::get/none"  (:wat::core::match (:wat::vector::get pv0 99)
                                           ((:wat::core::Some __x) false)
                                           (:wat::core::None true)))
      (:user::check "vector::conj"      (:wat::core::= (:wat::vector::length pv1) 4))
      (:user::check "vector::concat"    (:wat::core::= (:wat::vector::length pv2) 5))

      ;; ── Vector, 7 verbs: concat, conj, contains?, empty?, extend, get, length ──────────
      (:wat::core::let
        [v0    (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
         v-e   (:wat::core::Vector :- [:wat::core::i64])
         v1    (:wat::vec::conj v0 4)
         v2    (:wat::vec::concat v0 (:wat::core::Vector :- [:wat::core::i64] 4 5))
         v3    (:wat::vec::extend v0 pv0)]
        (:wat::core::do
          (:user::check "vec::length"    (:wat::core::= (:wat::vec::length v0) 3))
          (:user::check "vec::empty?/t"  (:wat::vec::empty? v-e))
          (:user::check "vec::empty?/f"  (:wat::core::not (:wat::vec::empty? v0)))
          (:user::check "vec::contains?" (:wat::vec::contains? v0 2))
          (:user::check "vec::get/some"  (:wat::core::match (:wat::vec::get v0 0)
                                            ((:wat::core::Some x) (:wat::core::= x 1))
                                            (:wat::core::None false)))
          (:user::check "vec::get/none"  (:wat::core::match (:wat::vec::get v0 99)
                                            ((:wat::core::Some __x) false)
                                            (:wat::core::None true)))
          (:user::check "vec::conj"      (:wat::core::= (:wat::vec::length v1) 4))
          (:user::check "vec::concat"    (:wat::core::= (:wat::vec::length v2) 5))
          (:user::check "vec::extend"    (:wat::core::= (:wat::vec::length v3) 6))
          (:wat::kernel::println "ALL 13 OK"))))))
