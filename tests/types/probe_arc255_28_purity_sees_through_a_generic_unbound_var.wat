;; Stone 255.28 — an UNBOUND type variable stays decided at instantiation: a generic `Record`
;; whose field is `(E :- [T])` substitutes `T` into `:Started` as `(Address :- [i64 i64 T])`,
;; and `T` is a formal parameter, not a Shared marker: accepted. The instantiation that makes it
;; impure was _unbound_var_instantiated_shared.wat.bad; 255.29 inverts that twin to
;; an accepted `.wat` because a Shared address is data.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defrecord :probe::Carrier :- [T]
  [status <- (:probe::E :- [T])])
(:wat::core::defrecord :probe::HoldsCarrier
  [carrier <- (:probe::Carrier :- [:wat::kernel::Transport.Wire])])
