;; The arc-294 R2 homecoming cosines — plain EDN measured directly (294.a).
;; No manual (to-holon …): the measurement surface now lifts any EdnRepresentable
;; value internally. Expected, structurally: 1.0 / ~0.486 / ~0.574 / ~0.011.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; identical map → exact coincidence
    (:wat::kernel::pprintln (:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 2}))
    ;; one of two role-filler binds matches → ~½
    (:wat::kernel::pprintln (:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 3}))
    ;; two of three positional binds match
    (:wat::kernel::pprintln (:wat::holon::cosine [1 2 3] [1 2 4]))
    ;; share nothing → near-orthogonal in hyperspace
    (:wat::kernel::pprintln (:wat::holon::cosine {:a 1 :b 2} {:zzz :qqq}))
    nil))
