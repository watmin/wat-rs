;; Arc 255 Stone 2a — ⛔ THE DEFECT THIS STONE MINTED, read back from the registry itself.
;;
;; `:wat::rete::i64::>` is an ALIAS of `:wat::i64::>` — same behaviour, by construction,
;; because the registry re-dispatches one to the other. Yet each row declares its own five
;; axes, and they DISAGREE on two of them. Two names, one behaviour, contradicting each
;; other about what that behaviour is.
;;
;; This is the exact drift the RULING exists to eliminate, minted inside the campaign, by
;; the stone that added the alias field — and it appeared the instant an alias was allowed
;; to declare axes instead of inheriting them.
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:wat::core::render-doc :wat::rete::i64::>))
    (:wat::kernel::println (:wat::core::render-doc :wat::i64::>))))
