;; tests/comms/probe_arc293_W2a_struct_no_cross.wat
;; Co-located fixture for probe_arc293_W2a_struct_no_cross.rs (startup_beside).
;;
;; Arc 293.W.2a — the struct wire-wall (both directions, §7).
;;
;; INBOUND (recv' backstop, decode_trusted_wire):
;; probe-struct: child pprintln's a bare Nature::Struct over the process peer's
;;   stdout; parent recv's it.
;;   Before fix: decode succeeds (breach open) → i64(99) returned.
;;   After fix:  decode_trusted_wire refuses it (breach closed) → RuntimeError.
;; probe-record: child pprintln's a base Record; parent recv's it.
;;   Must succeed at HEAD AND after the fix (records are wire-portable).
;;
;; OUTBOUND (send' guard, eval_peer_send_prime):
;; probe-send-struct: parent send's a bare struct to a PROCESS child.
;;   Before fix: serializes (breach open) → nil. After fix: guard refuses → RuntimeError.
;; probe-send-record: parent send's a base record to a PROCESS child.
;;   Must succeed both directions (records are portable) → nil.
;; probe-send-struct-thread: parent send's a struct over a THREAD peer (in-locus,
;;   no serialization) and recv's it back. Must succeed — the thread tier is NOT
;;   guarded (a struct over a thread peer is legitimate; same address space).

(:wat::core::defstruct :w2a::S [val <- :wat::core::i64])
(:wat::core::defrecord :w2a::R [val <- :wat::core::i64])

;; Struct probe — sends a bare struct over the wire.
(:wat::core::defn :w2a::probe-struct [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defstruct :w2a::S [val <- :wat::core::i64])
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::pprintln (:w2a::S 99)))))]
    (:w2a::S/val (:wat::kernel::recv' p))))

;; Record control probe — sends a base record over the wire.
(:wat::core::defn :w2a::probe-record [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defrecord :w2a::R [val <- :wat::core::i64])
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::pprintln (:w2a::R 42)))))]
    (:w2a::R/val (:wat::kernel::recv' p))))

;; ── OUTBOUND: send' guard ─────────────────────────────────────────
;;
;; Arc 293.W.2c (compile-time supersession): probe-send-struct (typed struct → process peer)
;; is now REJECTED BY THE TYPE-CHECKER (infer_send_prime portability gate). Including it in
;; this world would cause startup to fail, breaking the inbound and thread-control tests above.
;; It lives in tests/comms/probe_arc293_W2c_compile_time_send.wat and is tested there.
;; The Rust test struct_rejected_at_wire_SEND below points to that probe via startup_from_file.

;; Send-record control — parent send's a base record to a PROCESS child.
;; Must succeed (records are portable): send' returns nil. The child reads the
;; line raw as a String (no decode crash), keeping stdin open for the write.
(:wat::core::defn :w2a::probe-send-record [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [_ (:wat::kernel::readln -> :wat::core::String)] nil))))
     _ (:wat::kernel::send' p (:w2a::R 42))]
    nil))

;; Thread control — a struct over a THREAD peer round-trips in-locus (no
;; serialization, no guard). The thread self-peer echoes the struct back; the
;; parent extracts the field. Proves the send' guard is process/socket-only.
(:wat::core::defn :w2a::probe-send-struct-thread [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<w2a::S,w2a::S>] -> :wat::core::nil
              (:wat::kernel::send' self (:wat::kernel::recv' self))))
     _   (:wat::kernel::send' peer (:w2a::S 99))
     got (:wat::kernel::recv' peer)]
    (:w2a::S/val got)))
