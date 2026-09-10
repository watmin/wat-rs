;; Co-located fixture for peer_verb_round_trip_process.rs — slurped via startup_beside(file!()).
;; #[ignore] process-tier probe (arc 214 Stone 4.6a-ii).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [n (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])
                   _ (:wat::kernel::println (:wat::i64::+ n 1))]
                  nil))))
     _   (:wat::core::match (:wat::kernel::send peer 41) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil] [:wat::kernel::SendOutcome.Stopped {} nil]) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     got (:wat::core::match (:wat::kernel::recv peer)
           [:wat::kernel::RecvOutcome.Message {:msg m} m]
           [:wat::kernel::RecvOutcome.Lost {:cause cause}
             (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
           [:wat::kernel::RecvOutcome.Stopped {}
             (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
           [:wat::kernel::RecvOutcome.Closed {}
             (:wat::kernel::assertion-failed! :message "recv': process peer closed before replying")])]
    got))

