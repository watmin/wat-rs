;; Stone 255.28, inverted by 255.29. CONTROL A (record field): the bare Shared address
;; as a pure `Record` field. Pre-255.29 this file was `.wat.bad` and froze as
;; ImpureFieldInPureAggregate (`is_pure_type`'s Address arm keyed Shared to false).
;; 255.29: a Shared address is data (ThreadAddressWire), so the field is pure.
(:wat::core::defrecord :probe::HoldsBare
  [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])])
