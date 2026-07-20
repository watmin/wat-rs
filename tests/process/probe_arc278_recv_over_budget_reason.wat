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
(:wat::core::defn :user::over-budget-recv [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process/max-message-bytes 256)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println
               "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"))))
     _ (:wat::kernel::recv' p)]
    0))
