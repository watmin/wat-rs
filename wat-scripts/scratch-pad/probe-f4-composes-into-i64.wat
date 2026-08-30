;; FINDING F4 composed — checker binds numerator as i64; runtime produced bigint; i64::+ dies.
;;
;; check.rs:17048  rational/numerator : rational -> i64   -- "runtime returns bigint
;;                 for a component that overflows i64"
;;
;; Independent of F3 (no rational-op collapse). The accessor itself is the optimistic
;; declaration. `--check` is SILENT; the runtime raises TypeMismatch:
;;   :wat::core::i64::+: expected i64, got wat::core::bigint `18446744073709551614N`
;;
;; ⚠ THIS PROBE RAISES AT RUNTIME BY DESIGN. It type-checks, which is the finding.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [big (:wat::core::bigint::* (:wat::core::i64::to-bigint 9223372036854775807)
                                (:wat::core::i64::to-bigint 2))
     r   (:wat::core::rational::/ (:wat::core::bigint::to-rational big)
                                   (:wat::core::i64::to-rational 3))
     n   (:wat::core::rational/numerator r)]
    (:wat::kernel::println (:wat::edn::write (:wat::core::i64::+ n 1)))))
