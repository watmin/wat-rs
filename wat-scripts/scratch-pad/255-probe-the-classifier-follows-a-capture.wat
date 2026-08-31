;; Scratch probe — arc 255 Stone A-2-i: THE CLASSIFIER MAY HOLD AN ENVIRONMENT.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-2-i-the-classifier-may-hold-an-environment.md
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-i-the-classifier-may-hold-an-environment.md
;;
;; `head_ok` (`src/rete/purity.rs`) now resolves a bare call head naming a LOCAL BINDING through
;; `ClassifyCtx::Runtime(env)`: `:wat::rete::pure?`/`deterministic?`/`total?` pass their OWN `env`
;; down. Both rows below are load-bearing:
;;
;;   row 1 — `keyfn` is PURE,      bound in an enclosing `let`  -> true   (was `false` before this stone)
;;   row 2 — `keyfn` is EFFECTFUL, bound the same way           -> false (proves the capability did
;;                                                                        NOT widen into an
;;                                                                        always-true classifier)
;;
;; The sibling negative control, `255-probe-the-classifier-cannot-see-through-a-closure.wat` (a
;; comparator whose `keyfn` is bound NOWHERE), must still print `true` / `false` / `false` —
;; unchanged by this stone.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; row 1 — keyfn PURE, bound in scope -> true
    (:wat::kernel::println
      (:wat::core::let [keyfn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                                 (:wat::core::* x 2))]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
            (:wat::core::< (keyfn a) (keyfn b)))))))
    ;; row 2 — keyfn EFFECTFUL, bound in scope -> false (no widening)
    (:wat::kernel::println
      (:wat::core::let [keyfn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                                 (:wat::core::do (:wat::kernel::println "!") x))]
        (:wat::rete::pure? (:wat::core::quote
          (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
            (:wat::core::< (keyfn a) (keyfn b)))))))))
