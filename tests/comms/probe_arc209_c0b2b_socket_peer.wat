;; tests/comms/probe_arc209_c0b2b_socket_peer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b.2b — socket-backed Peer' + socket-pair'. Mint a connected pair, round-trip 5.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::socket-pair' :wat::core::i64 :wat::core::i64)
     a    (:wat::core::first pair)
     b    (:wat::core::second pair)
     _    (:wat::kernel::send' a 5)
     got  (:wat::kernel::recv' b)]
    got))

