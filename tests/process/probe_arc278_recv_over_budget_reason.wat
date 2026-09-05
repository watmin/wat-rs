;; Arc 278 #15 (mute-kill floor) — the SPEAK facet at the wat `recv'` client surface, PROCESS locus.
;;
;; The parent spawns a child via `spawn-program'` with a DELIBERATELY SMALL per-message frame budget
;; (`process/max-message-bytes 256`). The child `println`s ONE string whose EDN-framed size EXCEEDS the
;; budget. The parent `recv'`s it. The parent's output receiver (fd 1, budgeted) rejects the over-budget
;; frame with `RecvError::FrameTooLarge`.
;;
;; THE LAW (wat never hides a failure): the `recv'` MUST raise carrying the frame-cap reason
;; ("frame exceeded cap (message larger than the receiver's max-message-bytes budget)"), NOT collapse to
;; the reasonless mute "recv failed: peer closed / channel disconnected".
;;
;; The co-located `.rs` calls this entry fn and asserts the raised message CONTAINS the frame-cap reason
;; and does NOT read as the bare peer-closed mute.

;; Child body helper — a single over-budget payload (400 'X' chars → EDN-framed well past the 256-byte cap).
;; Arc 278 recv'-wall: recv' returns a matchable RecvOutcome VALUE (never a raise). The parent's
;; budgeted receiver rejects the over-budget frame (RecvError::FrameTooLarge) → the outcome is ::Lost
;; carrying the frame-cap reason. We MATCH and RETURN the Lost cause's `Failure/message` as a VALUE the
;; .rs asserts (it must NAME the cap reason, not collapse to the reasonless peer-closed mute).
(:wat::core::defn :user::over-budget-recv [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process/max-message-bytes 256)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m) "UNEXPECTED-MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::LociDiedError/message cause))
      (:wat::kernel::RecvOutcome::Stopped "UNEXPECTED-STOPPED")
      (:wat::kernel::RecvOutcome::Closed "UNEXPECTED-CLOSED") (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
