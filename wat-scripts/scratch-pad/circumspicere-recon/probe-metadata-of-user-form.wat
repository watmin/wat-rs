;; Scratch probe (circumspicere recon, read-only) — mirrors the fixture used by
;; the #[ignore]d test `user_form_carries_guaranteed_baseline`
;; (tests/reflection/probe_arc255_reflection_parity.rs /
;; probe_arc255_reflection_parity_user_form.wat). Checking whether its
;; "not yet built" ignore reason still holds for a bare user `defn`.
(:wat::core::defn :my::f [x <- :wat::core::i64] -> :wat::core::i64 x)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m (:wat::runtime::metadata-of :my::f)]
    (:wat::core::match m
      ((:wat::core::Some _) (:wat::kernel::println "SOME (user defn carries baseline)"))
      (:wat::core::None (:wat::kernel::println "NONE (ignore reason still true)")))))
