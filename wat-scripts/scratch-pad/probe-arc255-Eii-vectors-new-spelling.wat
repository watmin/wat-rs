;; arc 255 Stone E-ii — sanity probe for the NEW spellings, phase 1 (register).
;; Not the acceptance probe (that lives at the end, exercising all 6+7 verbs); this is a
;; quick smoke test that the intrinsic registry actually dispatches both new namespaces.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [pv (:wat::vector::conj (wat.type/PersistentVector :- [wat.type/i64] 1) 2)
     v  (:wat::vec::conj (wat.type/Vector :- [wat.type/i64] 10) 20)
     total (:wat::core::+ (:wat::vector::length pv) (:wat::vec::length v))]
    (:wat::core::if (:wat::core::= total 4)
      (:wat::kernel::println "OK")
      (:wat::kernel::assertion-failed! :message (:wat::string::concat "expected 4, got " (:wat::core::str total))))))
