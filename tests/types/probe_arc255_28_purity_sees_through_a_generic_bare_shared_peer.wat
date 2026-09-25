;; Stone 255.28, inverted by 255.29. CONTROL A (peer payload): a wire peer produced
;; over the bare Shared address. Pre-255.29 this file was `.wat.bad` and the §7 peer
;; wall refused it. 255.29: a Shared address is data, so the payload is pure.
(:wat::core::defn :probe::bare-peer [] -> :wat::core::nil
  (:wat::core::let
    [_self (:wat::program::self-peer
             (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])
             :wat::core::i64)]
    nil))
