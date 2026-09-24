;; Stone 255.25 (C-b4) — POSITIVE twin of _address_shared_field.wat.bad: the same address on
;; the WIRE transport is portable, so it is a pure `Record` field.
(:wat::core::defrecord :probe::HoldsWire
  [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])])
