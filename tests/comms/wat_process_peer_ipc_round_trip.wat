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
