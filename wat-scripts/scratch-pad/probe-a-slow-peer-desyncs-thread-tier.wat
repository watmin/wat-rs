;; PROBE — does the fired-deadline desync exist on the THREAD tier too?
;; Same shape as the process-tier probe; only the locus differs. Decides whether stone 2
;; (DeadlineFired redials) can repair the thread tier at all, since a thread peer has no address.
;;
;; `the-gate-methods-face-an-outcome/DESIGN.md` ruled re-recv over re-ask to avoid a surplus reply
;; left on the wire "if the first ack was merely SLOW rather than lost", calling it
;; "character-for-character the frame-desync class that produced today's crash". Nothing in this tree
;; can produce a slow peer, so that hazard has never been observed. This probe tries to produce it.
;;
;; THE THREE THINGS IT MUST SHOW, in order:
;;   1. a handler CAN delay without a sleep (timer channel, mora-legal, inlined because a forked
;;      child cannot see parent helpers — the publisher SCORE's lesson)
;;   2. a short deadline against that slow handler yields DeadlineFired while the service LIVES
;;   3. ⭑ the NEXT call on the same peer — does it get its own reply, or call 1's stale one?
;;
;; ⛔ Every observation PRINTS. A probe that raises on the thing it measures reports nothing.
(:wat::core::defsurface :slow::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :slow::Svc::GoRequest [tag <- :wat::core::i64])
   (:wat::core::defenum :slow::Svc::GoResponse :wat::enum::Pure
     :Ok               [tag <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(go [self <- :slow::Svc  req <- :slow::Svc::GoRequest]
     -> :slow::Svc::GoResponse :max-request-bytes 4096)])

(:wat::service::defservice :slow::svc
  :satisfies :slow::Svc
  :durable   [delay-ms <- :wat::core::i64]
  :ephemeral []
  :impls
  [(go [s ctx req]
     ;; THE DELAY — inlined `recv (after …)`, not a sleep and not a parent helper.
     ;; A service is a serializing actor, so parking here makes THIS PEER SLOW, which is
     ;; exactly the fault being simulated.
     (:wat::core::let
       [_ (:wat::core::match
            (:wat::kernel::recv
              (:wat::kernel::after :wat::program::PeerKind::thread
                (:wat::time::Milliseconds (:slow::svc::Record/delay-ms (:slow::svc::State/durable s)))
                :done))
            ((:wat::kernel::RecvOutcome::Message _m) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil)
            (:wat::kernel::RecvOutcome::TimedOut nil)
            ((:wat::kernel::RecvOutcome::Malformed _c) nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:slow::Svc::Reply::Go (:slow::Svc::GoResponse::Ok (:slow::Svc::GoRequest/tag req))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:slow::Svc::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:slow::svc::Op])]))))])

;; One deadline-bounded call, reported. `tag` is the identity that makes a STALE reply visible.
(:wat::core::defn :slow::call
  [c <- (:wat::kernel::Peer :- [:slow::Svc::Op :slow::Svc::Reply])
   label <- :wat::core::String  tag <- :wat::core::i64  ms <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::service::call-by-deadline c
      (:slow::Svc::Op::Go (:slow::Svc::GoRequest :tag tag)) ms
      (:slow::Svc::Reply::Go (:slow::Svc::GoResponse::Ok -1)))
    ((:wat::service::CallOutcome::Answered reply)
      (:wat::core::match reply
        ((:slow::Svc::Reply::Go resp)
          (:wat::core::match resp
            ((:slow::Svc::GoResponse::Ok got)
              (:wat::kernel::println
                (:wat::core::format "{l}=Answered(tag={g}) sent={s}" :l label :g (:wat::i64::to-string got) :s (:wat::i64::to-string tag))))
            ((:slow::Svc::GoResponse::RequestTooLarge b cap)
              (:wat::kernel::println (:wat::string::concat label "=TooLarge")))
            ((:slow::Svc::GoResponse::RequestMalformed p e g)
              (:wat::kernel::println (:wat::string::concat label "=RequestMalformed")))))
        ;; the synthesized transport variant on every <S>::Reply — a decode fact, not an op fact
        ((:slow::Svc::Reply::Failed cause)
          (:wat::kernel::println (:wat::string::concat label "=Reply::Failed")))))
    ((:wat::service::CallOutcome::Lost c2) (:wat::kernel::println (:wat::string::concat label "=Lost")))
    ((:wat::service::CallOutcome::Closed) (:wat::kernel::println (:wat::string::concat label "=Closed")))
    ((:wat::service::CallOutcome::DeadlineFired) (:wat::kernel::println (:wat::string::concat label "=DeadlineFired")))
    ((:wat::service::CallOutcome::Malformed _c) (:wat::kernel::println (:wat::string::concat label "=Malformed")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:slow::svc/start :locus (:wat::spawn::thread) :record (:slow::svc::Record :delay-ms 600))
     c (:wat::core::match (:wat::kernel::connect (:slow::svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "dial refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "dial rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "dial failed" :wat::core::None :wat::core::None)))
     ;; (1) deadline 100 ms against a 600 ms handler → the caller gives up FIRST
     _ (:slow::call c "call-1(tag=11,dl=100)" 11 100)
     ;; (2) ⭑ THE MEASUREMENT: a second call, generous deadline. If it returns tag=11,
     ;;     call 1's stale reply was sitting on the wire and this call read it.
     _ (:slow::call c "call-2(tag=22,dl=5000)" 22 5000)
     ;; (3) non-vacuity: a third call must still be sane
     _ (:slow::call c "call-3(tag=33,dl=5000)" 33 5000)
     ;; (4) ⭑ THE GENERATED CLIENT METHOD on the SAME desynced peer. service.wat:2714 maps
     ;;     DeadlineFired -> RecvOutcome::TimedOut and hands the peer back unchanged, so the
     ;;     macro-generated path inherits the shift rather than repairing it.
     _ (:wat::core::match (:slow::Svc/go c (:slow::Svc::GoRequest :tag 44))
         ((:wat::kernel::RecvOutcome::Message resp)
           (:wat::core::match resp
             ((:slow::Svc::GoResponse::Ok got)
               (:wat::kernel::println
                 (:wat::core::format "call-4-GENERATED=Ok(tag={g}) sent=44" :g (:wat::i64::to-string got))))
             ((:slow::Svc::GoResponse::RequestTooLarge b cap) (:wat::kernel::println "call-4-GENERATED=TooLarge"))
             ((:slow::Svc::GoResponse::RequestMalformed p e g) (:wat::kernel::println "call-4-GENERATED=RequestMalformed"))))
         ((:wat::kernel::RecvOutcome::Lost c2) (:wat::kernel::println "call-4-GENERATED=Lost"))
         (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "call-4-GENERATED=Stopped"))
         (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "call-4-GENERATED=Closed"))
         (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println "call-4-GENERATED=TimedOut"))
         ((:wat::kernel::RecvOutcome::Malformed c2) (:wat::kernel::println "call-4-GENERATED=Malformed")))]
    nil))
