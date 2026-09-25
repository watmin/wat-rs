;; Stone 255.28, inverted by 255.29. SUBJECT B (peer payload): the Shared address
;; through the `Pure` generic enum, as a wire peer's payload. 255.28 refused it
;; at the §7 wall. Pre-255.29 this file was `.wat.bad`. 255.29: the address is
;; data, so the payload is pure.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defn :probe::generic-peer [] -> :wat::core::nil
  (:wat::core::let
    [_self (:wat::program::self-peer (:probe::E :- [:wat::kernel::Transport.Shared]) :wat::core::i64)]
    nil))
