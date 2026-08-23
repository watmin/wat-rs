;; probe-s1-fn-forms.wat — RED probe / acceptance target for 259 S1 (expose closure_extract as fn-forms).
;;
;; CLAIM: (:wat::kernel::fn-forms f name) reifies a fn (anonymous OR named) into self-contained
;; forms that (def name <the-fn>) + its transitive deps in a FRESH universe, ImpureCapture-gated.
;; The not-shared bracket path calls this to ship the work-fn across a fork.
;;
;; This routes the closure-seam through fn-forms: reify an anon block → ship the forms to a process
;; worker → stream. RED at HEAD (fn-forms does not exist → UnknownFunction). GREEN once S1 lands: "6 10".

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; the work-fn as a runtime anonymous block (Ruby's Parallel { |x| x*2 })
     work       (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
     ;; reify it to shippable forms that define it under :probe::work in the child's fresh universe
     work-forms (:wat::kernel::fn-forms work :probe::work)
     ;; assemble the child program: the reified work FIRST (so :probe::work resolves), then the
     ;; runner + child-main that reference it.
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat
           work-forms
           (:wat::core::forms
             (:wat::core::defn :probe::runner
               [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
               (:wat::core::let
                 [item (:wat::kernel::recv self)
                  _    (:wat::core::match (:wat::kernel::send self (:probe::work item)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                 (:probe::runner self)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:probe::runner (:wat::program::self-peer :wat::core::i64 :wat::core::i64))))))
     _ (:wat::core::match (:wat::kernel::send w 3) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     _ (:wat::core::match (:wat::kernel::send w 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::i64::to-string a)
        (:wat::core::string::concat " " (:wat::core::i64::to-string b))))))
