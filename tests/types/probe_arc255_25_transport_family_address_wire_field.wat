;; Stone 255.25 (C-b4) — the WIRE twin of _address_shared_field.wat. Both transports
;; are pure Record fields after 255.29; Wire was already pure before it.
(:wat::core::defrecord :probe::HoldsWire
  [addr <- (:wat::kernel::Address :- [wat.type/i64 wat.type/i64 :wat::kernel::Transport.Wire])])
