;; arc 278 the recv'-outcome wall — STOP-0 proof (the HARD GATE).
;;
;; Prove a `RecvOutcome::Lost(cause)` carries the STRUCTURED crash cause end-to-end
;; on the REAL path: a thread peer whose body crashes; the owner recv's it and MATCHES
;; `((RecvOutcome::Lost cause) …)` — a VALUE, not a raise — and the cause is a
;; `:wat::kernel::Failure` whose `/message` contains the crash sentinel.
;;
;; The cause is built via the SAME structured carrier `ServiceEvent::Lost` uses
;; (message_only_failure over the crash-channel reason) — no crash_tx reshape needed.
;; Runs to stdout: prints "STOP0-LOST-MESSAGE: …BOOM-SENTINEL-9173…" on success; any
;; other arm eprintln's (terminal, exits non-zero).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "BOOM-SENTINEL-9173" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::kernel::recv p) 
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::eprintln "STOP0-FAIL: got RecvOutcome::Message, expected ::Lost"))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println
          (:wat::string::concat "STOP0-LOST-MESSAGE: " (:wat::kernel::LociDiedError/message cause))))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::eprintln "STOP0-FAIL: got RecvOutcome::Stopped, expected ::Lost"))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::eprintln "STOP0-FAIL: got RecvOutcome::Closed, expected ::Lost")))))
