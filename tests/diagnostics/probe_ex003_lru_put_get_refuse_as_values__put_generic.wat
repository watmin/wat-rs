;; Fixture for probe_ex003_lru_put_get_refuse_as_values.rs — `Lru/put` keyed on an opaque handle,
;; LAUNDERED through a generic `K`: `:user::put-any` never names the handle's type, so no
;; call-site annotation could have caught it. The refusal must still arrive as an `Err` value
;; naming `:wat::cache::Lru/put`.
(:wat::core::defn :user::put-any :- [K]
  [cache <- (:wat::cache::Lru :- [K :wat::core::i64])
   k     <- :K]
  -> :wat::core::String
  (:wat::core::match (:wat::cache::Lru/put cache k 1)
    [:wat::core::Result.Ok {:value _displaced} "UNREFUSED"]
    [:wat::core::Result.Err {:error fault}
      (:wat::string::concat
        (:wat::cache::Fault/diagnostic fault)
        (:wat::string::concat " | " (:wat::cache::Fault/message fault)))]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [key   (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "key cache")
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64]) :wat::core::i64] 2) "outer cache")]
    (:wat::kernel::println (:user::put-any cache key))))
