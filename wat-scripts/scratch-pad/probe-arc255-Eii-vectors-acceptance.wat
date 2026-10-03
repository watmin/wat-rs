;; arc 255 Stone E-ii — the acceptance probe: every verb of BOTH families, under the new
;; spellings, asserts a concrete result. 6 PersistentVector verbs + 7 Vector verbs = 13.
(:wat::core::defn :user::check [label <- wat.type/String cond <- wat.type/bool] -> wat.type/nil
  (:wat::core::if cond
    (:wat::kernel::println (:wat::string::concat "ok   " label))
    (:wat::kernel::assertion-failed! :message (:wat::string::concat "FAIL " label))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    ;; ── PersistentVector, 6 verbs: concat, conj, contains?, empty?, get, length ──────────
    [pv0     (wat.type/PersistentVector :- [wat.type/i64] 1 2 3)
     pv-e    (wat.type/PersistentVector :- [wat.type/i64])
     pv1     (:wat::core::conj pv0 4)
     pv2     (:wat::core::into pv0 (wat.type/PersistentVector :- [wat.type/i64] 4 5))]
    (:wat::core::do
      (:user::check "vector::length"    (:wat::core::= (:wat::core::length pv0) 3))
      (:user::check "vector::empty?/t"  (:wat::core::empty? pv-e))
      (:user::check "vector::empty?/f"  (:wat::core::not (:wat::core::empty? pv0)))
      (:user::check "vector::contains?" (:wat::core::contains? pv0 2))
      (:user::check "vector::get/some"  (:wat::core::match (:wat::core::get pv0 0)
                                           [:wat::core::Option.Some {:value x} (:wat::core::= x 1)]
                                           [:wat::core::Option.None {} false]))
      (:user::check "vector::get/none"  (:wat::core::match (:wat::core::get pv0 99)
                                           [:wat::core::Option.Some {:value __x} false]
                                           [:wat::core::Option.None {} true]))
      (:user::check "vector::conj"      (:wat::core::= (:wat::core::length pv1) 4))
      (:user::check "vector::concat"    (:wat::core::= (:wat::core::length pv2) 5))

      ;; ── Vector, 7 verbs: concat, conj, contains?, empty?, extend, get, length ──────────
      (:wat::core::let
        [v0    (wat.type/Vector :- [wat.type/i64] 1 2 3)
         v-e   (wat.type/Vector :- [wat.type/i64])
         v1    (:wat::core::conj v0 4)
         v2    (:wat::core::concat v0 (wat.type/Vector :- [wat.type/i64] 4 5))
         v3    (:wat::core::into v0 pv0)]
        (:wat::core::do
          (:user::check "vec::length"    (:wat::core::= (:wat::core::length v0) 3))
          (:user::check "vec::empty?/t"  (:wat::core::empty? v-e))
          (:user::check "vec::empty?/f"  (:wat::core::not (:wat::core::empty? v0)))
          (:user::check "vec::contains?" (:wat::core::contains? v0 2))
          (:user::check "vec::get/some"  (:wat::core::match (:wat::core::get v0 0)
                                            [:wat::core::Option.Some {:value x} (:wat::core::= x 1)]
                                            [:wat::core::Option.None {} false]))
          (:user::check "vec::get/none"  (:wat::core::match (:wat::core::get v0 99)
                                            [:wat::core::Option.Some {:value __x} false]
                                            [:wat::core::Option.None {} true]))
          (:user::check "vec::conj"      (:wat::core::= (:wat::core::length v1) 4))
          (:user::check "vec::concat"    (:wat::core::= (:wat::core::length v2) 5))
          (:user::check "vec::extend"    (:wat::core::= (:wat::core::length v3) 6))
          (:wat::kernel::println "ALL 13 OK"))))))
