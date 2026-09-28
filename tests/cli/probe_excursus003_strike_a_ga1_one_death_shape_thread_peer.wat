;; tests/cli/probe_excursus003_strike_a_ga1_one_death_shape_thread_peer.wat
;; GA1 (excursus 003 strike A) — the SAME claim through a thread peer: :user::main
;; spawns a thread peer whose body panics via assert-eq, then recv's and silently
;; absorbs the Lost outcome (returns nil) so the OVERALL process exits 0 — the ONE
;; stderr line this proves comes from the process-wide panic hook alone (the peer's
;; crash reason itself rides `crash_tx`, never stderr; see `wat::panic_hook`'s doc).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           (:wat::test::assert-eq 1 2)))]
    (:wat::core::match (:wat::kernel::recv p)
      [:wat::kernel::RecvOutcome.Message {:msg _m} nil]
      [:wat::kernel::RecvOutcome.Lost {:cause _c} nil]
      [:wat::kernel::RecvOutcome.Stopped {} nil]
      [:wat::kernel::RecvOutcome.Closed {} nil])))
