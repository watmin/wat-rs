;; tests/comms/probe_arc209_c0b2b_socket_peer.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b.2b — socket-backed Peer' + socket-pair'. Mint a connected pair, round-trip 5.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::socket-pair' :wat::core::i64 :wat::core::i64)
     a    (:wat::core::first pair)
     b    (:wat::core::second pair)
     _    (:wat::core::match (:wat::kernel::send' a 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     got  (:wat::core::match (:wat::kernel::recv' b)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': b closed unexpectedly" :wat::core::None :wat::core::None)))]
    got))

