;; FINDING F4 — rational/numerator declared i64; runtime returns bigint past i64.
;; Independently reproducing FINDINGS.md F4. Deleted-or-kept after SCORE.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [big (:wat::core::bigint::* (:wat::core::i64::to-bigint 9223372036854775807)
                                (:wat::core::i64::to-bigint 2))
     r   (:wat::core::rational::/ (:wat::core::bigint::to-rational big)
                                   (:wat::core::i64::to-rational 3))
     n   (:wat::core::rational/numerator r)]
    (:wat::kernel::println (:wat::edn::write n))))
