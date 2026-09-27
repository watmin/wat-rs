;; Excursus 003 step 4 (D4) — G5, the overflow inside a THREAD-spawned peer.
;;
;; A THREAD locus (`:wat::spawn::thread`) runs a function of the SAME already-frozen
;; world on a NEW OS thread — unlike a PROCESS locus, it never re-runs the load
;; pipeline. `:user::grow`'s overflow, reused verbatim from G1, fires inside the
;; thread's own body; the thread dies before it can send, and the owner's `recv`
;; surfaces the death as a matchable `LociDiedError`.
(:wat::core::defn :user::grow [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ n 9223372036854775807))

(:wat::core::defn :probe::overflow-in-thread [] -> :wat::kernel::LociDiedError
  (:wat::core::let
    [peer (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [_self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
              (:wat::core::let [_ (:user::grow 41)] nil)))
     r (:wat::kernel::recv peer)]
    (:wat::core::match r
      [:wat::kernel::RecvOutcome.Lost {:cause cause} cause]
      [:wat::kernel::RecvOutcome.Message {:msg _m}
        (:wat::kernel::assertion-failed! :message "expected the thread to die from the overflow, not send a message")]
      [:wat::kernel::RecvOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "recv: stopped — expected the thread to die from the overflow")]
      [:wat::kernel::RecvOutcome.Closed {}
        (:wat::kernel::assertion-failed! :message "recv: closed cleanly — expected a death from the overflow")])))
