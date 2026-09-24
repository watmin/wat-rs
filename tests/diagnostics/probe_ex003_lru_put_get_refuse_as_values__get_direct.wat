;; Fixture for probe_ex003_lru_put_get_refuse_as_values.rs — `Lru/get` keyed on an opaque handle.
;;
;; A key that cannot be stored cannot be present, so `get` MISSES — its return type is already
;; `Option`, and the answer is total. Precedent: `HashMap`'s `contains-key?` answers `false` for
;; an unhashable key, "never inserted" (`src/collection/eval.rs`). Before excursus 003 stone C
;; this was a Rust `panic!`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [key   (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "key cache")
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64]) :wat::core::i64] 2) "outer cache")]
    (:wat::core::match (:wat::cache::Lru/get cache key)
      [:wat::core::Option.Some {:value _v} (:wat::kernel::println "HIT")]
      [:wat::core::Option.None {} (:wat::kernel::println "MISS")])))
