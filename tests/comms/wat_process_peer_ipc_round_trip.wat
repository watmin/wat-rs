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

;; T2 — real-spawn round-trip (substrate-composition proof). Spawns a subprocess (the
;; server) whose :user::main does one readln -> String + one println; composes a
;; ProcessPeer<String,String> out of Receiver/from-pipe + Sender/from-pipe over the
;; server's stdout/stdin, then Process/println "hello" + Process/readln + drain-and-join.
;; Construction is verbose by design (feedback_verbose_is_honest): the three-step build
;; surfaces what the run-processes bracket macro hides. Reply must equal "hello".
(:wat::core::defn :my::round-trip-hello [] -> :wat::core::String
  (:wat::core::let
    [server (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [line (:wat::kernel::readln -> :wat::core::String)
                     _    (:wat::kernel::println line)]
                    nil))))
     rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
     tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
     peer (:wat::kernel::ProcessPeer :rx rx :tx tx)]
    (:wat::core::match (:wat::kernel::Process/println peer "hello")
      -> :wat::core::String
      ((:wat::core::Ok _)
        (:wat::core::match (:wat::kernel::Process/readln peer)
          -> :wat::core::String
          ((:wat::core::Ok reply)
            (:wat::core::let [_drained (:wat::kernel::Process/drain-and-join server)]
              reply))
          ((:wat::core::Err _chain)
            (:wat::kernel::assertion-failed! "Process/readln failed: subprocess died" :wat::core::None :wat::core::None))))
      ((:wat::core::Err _chain)
        (:wat::kernel::assertion-failed! "Process/println failed: subprocess died" :wat::core::None :wat::core::None)))))
