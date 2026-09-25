;; Stone 255.28 — the WIRE twin of _generic_shared_field.wat: the same generic enum over
;; `Transport.Wire` instantiates to a Wire address, which is data: accepted.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defrecord :probe::HoldsGenericWire
  [status <- (:probe::E :- [:wat::kernel::Transport.Wire])])
