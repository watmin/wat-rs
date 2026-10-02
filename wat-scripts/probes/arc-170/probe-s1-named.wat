;; DISPOSITION (255.75) — repair: `:probe::runner` bare-bound `(recv self)` as `i` instead of
;; matching the `RecvOutcome` — the same defect class 255.73's `probe-s3b-astsplice.wat` site-2
;; repaired. Repaired by matching `recv` directly (Message processes-and-recurses; Lost raises;
;; Stopped/Closed exit). Also now asserts its own claim.
(:wat::core::defn :my::adder [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ n 5))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [wf (:wat::kernel::fn-forms :my::adder :probe::work)
     w  (:wat::test::spawn-peer (:wat::spawn::process)
          (:wat::core::concat wf
            (:wat::core::forms
              (:wat::core::defn :probe::runner [self <- (:wat::kernel::Peer :- [wat.type/i64 wat.type/i64])] -> wat.type/nil
                (:wat::core::match (:wat::kernel::recv self)
                  [:wat::kernel::RecvOutcome.Message {:msg i}
                    (:wat::core::let [_ (:wat::core::match (:wat::kernel::send self (:probe::work i)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])] (:probe::runner self))]
                  [:wat::kernel::RecvOutcome.Lost {:cause cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                  [:wat::kernel::RecvOutcome.Stopped {} nil]
                  [:wat::kernel::RecvOutcome.Closed {} nil]))
              (:wat::core::defn :user::main [] -> wat.type/nil
                (:probe::runner (:wat::program::self-peer wat.type/i64 wat.type/i64))))))
     _ (:wat::core::match (:wat::kernel::send w 1) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil]) _ (:wat::core::match (:wat::kernel::send w 2) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat (:wat::i64::to-string a) (:wat::string::concat " " (:wat::i64::to-string b))))
      (:wat::test::assert-eq a 6)
      (:wat::test::assert-eq b 7))))
