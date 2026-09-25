;; Stone 255.25, inverted by 255.29. A Shared address is data (a thread address
;; has a portable form, ThreadAddressWire). It is a pure Record field, the same
;; as the Wire twin. Pre-stone this file was `.wat.bad` and froze as
;; ImpureFieldInPureAggregate.
(:wat::core::defrecord :probe::HoldsShared
  [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])])
