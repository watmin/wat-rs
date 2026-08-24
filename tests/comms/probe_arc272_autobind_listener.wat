;; tests/comms/probe_arc272_autobind_listener.wat — co-located fixture for the autobind listener probe,
;; slurped via startup_beside(file!()). No placeholder main — startup_beside loads defns only.

;; Process-tier autobind: no name arg — the listener MINTS its own kernel-unique address.
;; Same signature shape as the thread tier: (listener' host :S :R) -> (Bound :- [S R]).
(:wat::core::defn :user::go [] -> :wat::core::bool
  (:wat::core::let
    [b (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     l (:wat::spawn::Bound/listener b)
     a (:wat::spawn::Bound/address b)
     c (:wat::core::match (:wat::kernel::connect a) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))]
    ;; Reaching here means: the 3-arg autobind form type-checked, minted a real listener +
    ;; address (capability), and connect' dialed the minted address — all with NO fixed name.
    true))
