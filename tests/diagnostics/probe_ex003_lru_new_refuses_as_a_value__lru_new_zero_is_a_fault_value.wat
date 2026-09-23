;; Fixture for probe_ex003_lru_new_refuses_as_a_value.rs — the-little-wat F-084, the CURED shape.
;;
;; `(:wat::cache::Lru/new 0)` returns `(Result :- [(Lru :- [K V]) :wat::cache::Fault])`. A
;; non-positive capacity is an `Err` VALUE this program can match and print — not a process death.
;; The printed `diagnostic` is the load-bearing half: it names the USER-FACING verb
;; `:wat::cache::Lru/new`, never the internal `:rust::cache::Lru/new` shim. Naming the shim is
;; half of what F-084 reports.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 0)
    [:wat::core::Result.Ok {:value _cache} (:wat::kernel::println "UNREFUSED")]
    [:wat::core::Result.Err {:error fault}
      (:wat::kernel::println
        (:wat::string::concat
          (:wat::cache::Fault/diagnostic fault)
          (:wat::string::concat " | " (:wat::cache::Fault/message fault))))]))
