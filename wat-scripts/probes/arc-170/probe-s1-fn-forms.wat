;; probe-s1-fn-forms.wat — RED probe / acceptance target for 259 S1 (expose closure_extract as fn-forms).
;;
;; CLAIM: (:wat::kernel::fn-forms f name) reifies a fn (anonymous OR named) into self-contained
;; forms that (def name <the-fn>) + its transitive deps in a FRESH universe, ImpureCapture-gated.
;; The not-shared bracket path calls this to ship the work-fn across a fork.
;;
;; This routes the closure-seam through fn-forms: reify an anon block → ship the forms to a process
;; worker → stream. RED at HEAD (fn-forms does not exist → UnknownFunction). GREEN once S1 lands: "6 10".
;;
;; DISPOSITION (255.75) — repair: `:probe::runner` bare-bound `(recv self)` as if it returned the
;; payload directly, and its `send` match was missing `SendOutcome.Stopped` — the same defect
;; class 255.73's `probe-s3b-astsplice.wat` site-2 repaired. Repaired the same way: `recv` is
;; matched on its `RecvOutcome` (Message processes-and-recurses; Lost raises; Stopped/Closed
;; exit), and the `Stopped` arm added to the `send` match. Also now asserts its own claim.

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [;; the work-fn as a runtime anonymous block (Ruby's Parallel { |x| x*2 })
     work       (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::* x 2))
     ;; reify it to shippable forms that define it under :probe::work in the child's fresh universe
     work-forms (:wat::kernel::fn-forms work :probe::work)
     ;; assemble the child program: the reified work FIRST (so :probe::work resolves), then the
     ;; runner + child-main that reference it.
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat
           work-forms
           (:wat::core::forms
             (:wat::core::defn :probe::runner
               [self <- (:wat::kernel::Peer :- [wat.type/i64 wat.type/i64])] -> wat.type/nil
               (:wat::core::match (:wat::kernel::recv self)
                 [:wat::kernel::RecvOutcome.Message {:msg item}
                   (:wat::core::let
                     [_ (:wat::core::match (:wat::kernel::send self (:probe::work item)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])]
                     (:probe::runner self))]
                 [:wat::kernel::RecvOutcome.Lost {:cause cause}
                   (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                 [:wat::kernel::RecvOutcome.Stopped {} nil]
                 [:wat::kernel::RecvOutcome.Closed {} nil]))
             (:wat::core::defn :user::main [] -> wat.type/nil
               (:probe::runner (:wat::program::self-peer wat.type/i64 wat.type/i64))))))
     _ (:wat::core::match (:wat::kernel::send w 3) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     _ (:wat::core::match (:wat::kernel::send w 5) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
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
      (:wat::kernel::println
        (:wat::string::concat
          (:wat::i64::to-string a)
          (:wat::string::concat " " (:wat::i64::to-string b))))
      (:wat::test::assert-eq a 6)
      (:wat::test::assert-eq b 10))))
