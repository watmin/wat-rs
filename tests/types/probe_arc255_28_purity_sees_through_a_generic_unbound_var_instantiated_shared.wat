;; Stone 255.28, inverted by 255.29. _unbound_var.wat's generic `Carrier`,
;; instantiated over `Transport.Shared` and seen through two generic layers
;; (Carrier -> E -> Address). 255.28 refused it. Pre-255.29 this file was
;; `.wat.bad`. 255.29: the address is data, so the instantiation is pure.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defrecord :probe::Carrier :- [T]
  [status <- (:probe::E :- [T])])
(:wat::core::defrecord :probe::HoldsCarrier
  [carrier <- (:probe::Carrier :- [:wat::kernel::Transport.Shared])])
