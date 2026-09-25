;; Stone 255.28, inverted by 255.29. SUBJECT B (record field): the Shared address
;; reached through a `Pure` generic enum (the defservice `Status` shape). 255.28
;; substituted the argument into `:Started` and refused, because a Shared address
;; was impure. Pre-255.29 this file was `.wat.bad`. 255.29: that address is data,
;; so the instantiated field is pure.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defrecord :probe::HoldsGeneric
  [status <- (:probe::E :- [:wat::kernel::Transport.Shared])])
