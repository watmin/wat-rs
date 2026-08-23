;; probe-arc278-let-tail-service-reaped.wat — a SCOPING bug, reproduced minimally.
;;
;; THE INVARIANT (builder, 2026-07-25): "c must not get reaped until the let is
;; completely resolved — it's still bound in that scope." A `let` binding is live for
;; the ENTIRE let form, INCLUDING the tail expression.
;;
;; THE VIOLATION: a request issued from a let's TAIL position finds the service CLOSED.
;; The identical call from a let-BINDING slot is served.
;;
;;   "from let-BINDING => Message (served)"
;;   "from let-TAIL    => CLOSED"
;;   "from let-TAIL+h  => CLOSED"     ← referencing the Handle in the tail does NOT save it
;;   "TAIL-ONLY call   => CLOSED"     ← and it is not about having made a prior call
;;
;; WHY IT IS SERIOUS, beyond the awkward shape: this is the most natural way to write
;; service code —
;;     (let [h (svc/start …)  c (connect' (Handle/addr h))]  (do-the-work c))
;; — and the work in the tail is silently unreachable.
;;
;; AND IT MANUFACTURES A FALSE `Closed`. Per R53 the reason-free `RecvOutcome::Closed`
;; is producible ONLY from a genuine clean EOF; it means "the peer closed normally."
;; Here it means "your service was reaped out from under you." The outcome wall is
;; correct and is being told a lie — the mute-failure class returning not through a
;; missing variant but through a right variant carrying a wrong story.
;;
;; MECHANISM: UNKNOWN. A hypothesis that fits every observation — drops scheduled at
;; the end of the BINDING LIST rather than the end of the LET FORM — is NOT grounded;
;; do not inherit it as fact.

(:wat::core::defsurface :tl::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :tl::Bag::PutRequest [n <- :wat::core::i64])
   (:wat::core::defenum :tl::Bag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :tl::Bag  req <- :tl::Bag::PutRequest]
     -> :tl::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :tl::bag-svc
  :satisfies :tl::Bag  :durable [n <- :wat::core::i64]  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply s (:tl::Bag::PutResponse::Ok 1)))])

(:wat::core::defn :tl::try [c <- (:wat::kernel::Peer :- [:tl::Bag::Op :tl::Bag::Reply])
                           label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:tl::Bag/put c (:tl::Bag::PutRequest :n 1))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::kernel::println (:wat::core::string::concat label " => Message (served)")))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::core::string::concat label " => LOST")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println (:wat::core::string::concat label " => STOPPED")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::core::string::concat label " => CLOSED")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:tl::bag-svc/start :locus (:wat::spawn::thread) :record (:tl::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:tl::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))
     ;; CONTROL — the same call from a BINDING slot: served.
     _ (:tl::try c "from let-BINDING")]
    ;; THE VIOLATION — the same call from the TAIL: closed.
    (:tl::try c "from let-TAIL   ")))
