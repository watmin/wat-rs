;; Stone 255.29 — a Shared address inside a pure record is accepted.
;; Pre-stone the same shape was ImpureFieldInPureAggregate.
(:wat::core::defrecord :probe25529::HoldsShared
  [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])])
