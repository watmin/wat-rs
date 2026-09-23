;; Fixture for probe_ex003_lru_new_refuses_as_a_value.rs — the UNHANDLED refusal, one level up.
;;
;; Same shape as the `Lru/new` sibling, through the composite. EXPECTATIONS row 4: a caller that
;; passes `0` to `HolographicLru/new` must get a wat error, not a panic — which is precisely what
;; the propagate decision buys. The raise's message names `:wat::cache::Lru/new` because that is
;; the verb that refused.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [store (:wat::core::Result/expect
             (:wat::cache::HolographicLru/new (:wat::holon::filter-accept-any) 0)
             ":wat::cache::Lru/new refused the capacity: it must be positive")]
    (:wat::kernel::println (:wat::cache::HolographicLru/len store))))
