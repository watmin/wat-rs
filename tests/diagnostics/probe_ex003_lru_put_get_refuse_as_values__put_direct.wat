;; Fixture for probe_ex003_lru_put_get_refuse_as_values.rs — `Lru/put` keyed on an opaque handle,
;; the key type written out at the call site.
;;
;; An `Lru` handle is well-typed as a `K` (the checker admits it) but not hashable at run time.
;; Before excursus 003 stone C this was a Rust `panic!` from `src/rust_deps/cache.rs`; now `put`
;; returns `(Result :- [(Option :- [(Entry :- [K V])]) :wat::cache::Fault])` and the refusal is an
;; `Err` VALUE whose `diagnostic` names the verb the user typed, `:wat::cache::Lru/put`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [key   (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "key cache")
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64]) :wat::core::i64] 2) "outer cache")]
    (:wat::core::match (:wat::cache::Lru/put cache key 1)
      [:wat::core::Result.Ok {:value _displaced} (:wat::kernel::println "UNREFUSED")]
      [:wat::core::Result.Err {:error fault}
        (:wat::kernel::println
          (:wat::string::concat
            (:wat::cache::Fault/diagnostic fault)
            (:wat::string::concat " | " (:wat::cache::Fault/message fault))))])))
