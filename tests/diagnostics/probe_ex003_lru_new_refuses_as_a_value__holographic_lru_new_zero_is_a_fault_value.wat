;; Fixture for probe_ex003_lru_new_refuses_as_a_value.rs — the ONE contract decision, driven.
;;
;; `HolographicLru/new` has no guard of its own; it delegates to `Lru/new`. Excursus 003 stone A
;; ruled the `Result` PROPAGATES rather than stopping at the composite — swallowing it here would
;; put a caller passing `0` back to a death with no span, one level up. So this prints the SAME
;; `:wat::cache::Fault`, and its `diagnostic` still names `:wat::cache::Lru/new` (not
;; `HolographicLru/new`), because that is where the refusal actually came from.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::cache::HolographicLru/new (:wat::holon::filter-accept-any) 0)
    [:wat::core::Result.Ok {:value _store} (:wat::kernel::println "UNREFUSED")]
    [:wat::core::Result.Err {:error fault}
      (:wat::kernel::println
        (:wat::string::concat
          (:wat::cache::Fault/diagnostic fault)
          (:wat::string::concat " | " (:wat::cache::Fault/message fault))))]))
