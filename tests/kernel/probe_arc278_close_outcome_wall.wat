;; Co-located fixture for probe_arc278_close_outcome_wall.rs — the process-tier
;; Closed[Some(code)] case. `close'` is :wat::kernel::-restricted, so it cannot be
;; called from this :user:: fixture; the fixture only SPAWNS + returns the process
;; peer, and the Rust probe drives close' on it via eval_in_frozen (no check pass).
;;
;; #[ignore] process-tier probe (arc 278) — forks a real child; run under setsid+timeout.

;; A thread self-peer whose worker returns nil immediately (a clean exit). Returned
;; to the Rust probe, which close's it and asserts CloseOutcome::Closed[exit = None]
;; (a thread has no OS exit code — loci-agnostic, R32).
(:wat::core::defn :user::spawn-noop-thread [] -> (:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64])
  (:wat::test::spawn-peer (:wat::spawn::thread)
    (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
      nil)))

;; A :process forms-server whose :user::main returns nil immediately → the child
;; exits cleanly (status 0). Returned to the Rust probe, which close's it and asserts
;; CloseOutcome::Closed[exit = Some(0)].
(:wat::core::defn :user::spawn-noop-process [] -> (:wat::kernel::Process :- [:wat::core::i64 :wat::core::i64])
  (:wat::test::spawn-peer (:wat::spawn::process)
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil nil))))
