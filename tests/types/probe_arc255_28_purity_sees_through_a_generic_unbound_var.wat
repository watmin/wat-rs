;; Stone 255.28 — an UNBOUND type variable stays decided at instantiation: a generic `Record`
;; whose field is `(E :- [T])` substitutes `T` into `:Started` as `(Address :- [i64 i64 T])`,
;; and `T` is a formal parameter, not a Shared marker: accepted. The instantiation that makes it
;; impure is _unbound_var_instantiated_shared.wat.bad.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defrecord :probe::Carrier :- [T]
  [status <- (:probe::E :- [T])])
(:wat::core::defrecord :probe::HoldsCarrier
  [carrier <- (:probe::Carrier :- [:wat::kernel::Transport.Wire])])
