;; arc 278 the send'-outcome wall — Phase 1 foundation DISCONFIRMING PROBE
;; (DESIGN-send-outcome-wall.md / BRIEF-send-wall-foundation.md).
;;
;; Before this strike: `send'` to a gone peer RAISED a reason-free MalformedForm
;; ("peer already closed" / "send failed: channel disconnected") — a raise unwinds
;; past the caller, masking the failure (the send-side twin of the recv'-wall's mute).
;; After: `send'` returns a matchable `:wat::kernel::SendOutcome` VALUE — never raises
;; on a gone peer.
;;
;; `:wat::kernel::close'` is kernel-internal-only (RAII teardown — a `:user::` caller
;; is a CHECK error per tests/kernel/probe_arc259_s2d_internal_only_close.wat.bad), so
;; this probe cannot construct the "already closed" case directly from user code.
;; Instead it constructs the genuinely-dead-peer case deterministically (no race):
;; spawn a thread worker that immediately crashes; `recv'` synchronizes on the crash
;; (mirrors the proven recv'-wall STOP0 probe — by the time `recv'` returns
;; `RecvOutcome::Lost`, the worker thread has fully unwound and dropped its channel
;; ends). A SECOND `send'` to that now-guaranteed-dead peer is exactly the pre-strike
;; "channel disconnected" raise site — post-strike it returns `SendOutcome::Lost`, a
;; matchable value.
;;
;; Runs to stdout: prints "PROBE-PASS: SendOutcome::Lost ..." on success (::Closed is
;; also accepted — both are values, not raises, per the design's own "Closed or Lost"
;; acceptance); ::Sent or any raise is a FAIL (assertion-failed!, non-zero exit).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "SEND-WALL-PROBE-CRASH" :wat::core::None :wat::core::None)))
     ;; synchronize on the worker's death: recv' blocks until EOF + the crash reason
     ;; lands on the crash channel — by the time this returns, the worker has fully
     ;; unwound and dropped its ends (deterministic, no race).
     r1 (:wat::kernel::recv p)
     _  (:wat::core::match r1
          ((:wat::kernel::RecvOutcome::Message _m)
            (:wat::kernel::assertion-failed!
              "PROBE-FAIL: expected worker crash (RecvOutcome::Lost), got Message"
              :wat::core::None :wat::core::None))
          ((:wat::kernel::RecvOutcome::Lost _cause) nil)
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed!
              "PROBE-FAIL: expected worker crash (RecvOutcome::Lost), got Stopped"
              :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed!
              "PROBE-FAIL: expected worker crash (RecvOutcome::Lost), got Closed"
              :wat::core::None :wat::core::None)))
     ;; the worker is now guaranteed dead. Pre-strike this send' RAISED "send failed:
     ;; channel disconnected"; post-strike it returns a matchable SendOutcome value.
     outcome (:wat::kernel::send p 42)]
    (:wat::core::match outcome
      (:wat::kernel::SendOutcome::Sent
        (:wat::kernel::assertion-failed!
          "PROBE-FAIL: got SendOutcome::Sent to a dead peer — expected Closed/Lost"
          :wat::core::None :wat::core::None))
      (:wat::kernel::SendOutcome::Closed
        (:wat::kernel::println
          "PROBE-PASS: SendOutcome::Closed (a VALUE, not a raise) after send' to a dead peer"))
      ;; arc 278 #73 judgment call (flagged, not silently decided): the design's own
      ;; framing generalizes past "Closed or Lost" — EVERY terminal send' outcome is a
      ;; matchable value, never a raise, and Stopped is no exception. No stop is ever
      ;; requested in this probe (it only forces a worker crash), so this arm is
      ;; unreached in practice; it is accepted here on the same "a value, not a raise"
      ;; principle the other two arms assert, not re-litigated as a new PASS criterion.
      (:wat::kernel::SendOutcome::Stopped
        (:wat::kernel::println
          "PROBE-PASS: SendOutcome::Stopped (a VALUE, not a raise) after send' to a dead peer"))
      ((:wat::kernel::SendOutcome::Lost cause)
        (:wat::kernel::println
          (:wat::core::string::concat
            "PROBE-PASS: SendOutcome::Lost (a VALUE, not a raise): "
            (:wat::kernel::LociDiedError/message cause)))))))
