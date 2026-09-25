;; tests/comms/probe_arc293_W2d_positive.wat
;; Positive fixture for probe_arc293_W2d_peer_purity.rs.
;;
;; Arc 293.W.2d — positive cases that MUST type-check:
;;
;; 255.30 — the impure thread-peer arm is refused and lives in
;; tests/kernel/probe_arc255_30_struct_on_thread_peer.wat.bad. This file keeps the
;; pure self-peer, which must still load.

;; :wat::program::self-peer with pure types — must still type-check.
(:wat::core::defn :w2d_pos::probe-pure-wire-peer [] -> :wat::core::nil
  (:wat::core::let
    [_pair (:wat::program::self-peer :wat::core::i64 :wat::core::i64)]
    nil))
