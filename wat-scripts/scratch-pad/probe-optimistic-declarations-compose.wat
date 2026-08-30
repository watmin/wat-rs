;; FINDING F3 — two "sound-enough" declarations COMPOSE into a well-typed program that dies.
;;
;; check.rs:16994  rational::+ : (rational, rational) -> rational   -- "EVERY op here can
;;                 collapse to :wat::core::bigint at runtime"
;; check.rs:17048  rational/numerator : rational -> i64             -- declared optimistically
;;
;; Each is individually defensible under this file's stated posture ("check is the optimistic
;; primary gate; the runtime is the honest backstop"). NEITHER comment mentions the other.
;; `--check` is SILENT on the composition below; the runtime raises TypeMismatch:
;;   :wat::core::rational/numerator: expected rational, got wat::core::bigint `1N`
;;
;; A sound-enough gate is not closed under composition — and that closure is what a type
;; system is for. See ~/work/gen-tests/FINDINGS.md F3.
;;
;; ⚠ THIS PROBE RAISES AT RUNTIME BY DESIGN. It type-checks, which is the finding, so it
;; satisfies the wat-scripts load gate; it is deliberately not wired into any deftest.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [half (:wat::core::rational::/ (:wat::core::i64::to-rational 1)
                                   (:wat::core::i64::to-rational 2))
     one  (:wat::core::rational::+ half half)]
    (:wat::kernel::println (:wat::edn::write (:wat::core::rational/numerator one)))))
