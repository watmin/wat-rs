;; tests/comms/probe_arc293_W2c_compile_time_send.wat
;; Co-located fixture for probe_arc293_W2c_compile_time_send.rs (startup_beside).
;;
;; Arc 293.W.2d supersedes 2c: the wall moved from send' time to peer PRODUCER time.
;;
;; After 2d, the compile-time purity wall is at wire-peer PRODUCERS (peer-pair',
;; socket-pair', connect', accept', program-self-peer'). An impure type arg to a
;; wire peer producer is a compile-time check error (§7 purity wall).
;;
;; This fixture creates peer-pair' with a struct type arg — struct is impure
;; (Holder::Struct) — the purity check at the producer fires at CHECK TIME.
;; The world FAILS TO LOAD (startup_beside returns Err) with a check error
;; mentioning "pure", "struct", or "wire".
;;
;; This is the same invariant as 2c but enforced at the peer SHAPE level, not
;; at the send' call site. The 2c send'-gate was deleted in arc 293.W.2d.

(:wat::core::defstruct :w2c::S [val <- :wat::core::i64])

;; peer-pair' with a struct type arg is a CHECK ERROR (§7 purity wall).
;; The wire peer producer checks that I,O are pure; a struct is impure.
(:wat::core::defn :w2c::probe-impure-wire-peer [] -> :wat::core::nil
  (:wat::core::let
    [_pair (:wat::kernel::peer-pair' :w2c::S :wat::core::i64)]
    nil))
