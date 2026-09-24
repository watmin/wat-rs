;; Fixture for probe_ex003_lru_put_get_refuse_as_values.rs — `Lru/get` keyed on an opaque handle,
;; LAUNDERED through a generic `K`. It must MISS, exactly as the direct case does.
(:wat::core::defn :user::get-any :- [K]
  [cache <- (:wat::cache::Lru :- [K :wat::core::i64])
   k     <- :K]
  -> :wat::core::String
  (:wat::core::match (:wat::cache::Lru/get cache k)
    [:wat::core::Option.Some {:value _v} "HIT"]
    [:wat::core::Option.None {} "MISS"]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [key   (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "key cache")
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64]) :wat::core::i64] 2) "outer cache")]
    (:wat::kernel::println (:user::get-any cache key))))
