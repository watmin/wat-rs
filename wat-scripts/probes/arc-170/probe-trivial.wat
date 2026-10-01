;; probe-trivial.wat — smoke baseline: the simplest possible program (one defn, one println).
;; CLAIM (exit 0): a minimal defn + println program loads and runs to completion. There is no
;; computed value beyond the literal "ok" sentinel, so exit 0 is the whole proof.
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
