;; Stone 255.28 — the WIRE twin of _generic_shared_peer.wat: accepted.
(:wat::core::defenum :probe::E :- [T] :wat::enum::Pure
  :Started [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 T])])
(:wat::core::defn :probe::generic-wire-peer [] -> :wat::core::nil
  (:wat::core::let
    [_self (:wat::program::self-peer (:probe::E :- [:wat::kernel::Transport.Wire]) :wat::core::i64)]
    nil))
