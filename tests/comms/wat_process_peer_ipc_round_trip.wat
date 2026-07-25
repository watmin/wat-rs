;; tests/comms/wat_process_peer_ipc_round_trip.wat — co-located fixture for the process peer IPC
;; round-trip probe's type-mint test. No placeholder main — startup_beside loads defns only.

(:wat::core::defn :my::client-reads-i64-writes-string
  [_peer <- :wat::kernel::ProcessPeer<wat::core::i64,wat::core::String>]
  -> :wat::core::nil
  nil)

(:wat::core::defn :my::client-reads-string-writes-i64
  [_peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::i64>]
  -> :wat::core::nil
  nil)

;; T2 — real-spawn round-trip (peer-wire proof). Spawns a process peer (the server)
;; via spawn-program' (process) whose :user::main does one readln -> String + one
;; println; feeds the child's readln with send' "hello", then drains the reply off
;; the peer via recv'. Reply must equal "hello". Arc 278 IPC de-prime: the composed
;; primes replace the Receiver/from-pipe + Sender/from-pipe + ProcessPeer/new dance —
;; spawn-program' returns the peer directly. Proven shape: wat_hermetic_round_trip.wat,
;; t18_echo_doubled.wat.
(:wat::core::defn :my::round-trip-hello [] -> :wat::core::String
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [line (:wat::kernel::readln )
                   _    (:wat::kernel::println line)]
                  nil))))
     _ (:wat::core::match (:wat::kernel::send' peer "hello")
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv' peer)
      ((:wat::kernel::RecvOutcome::Message reply) reply)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "round-trip-hello: subprocess closed before replying" :wat::core::None :wat::core::None)))))
