;; MEASUREMENT for D1-a — does a raise ESCAPING AN OP HANDLER kill the service for everyone?
;;
;; ⭐ THIS FILE IS RUN BY THE FLOOR. Row `a_handler_raise_does_not_kill_the_service_for_everyone`
;; in `tests/probes/scratch_pad_manifest.rs` (excursus 001 `the-probes-run-in-the-floor`) runs it
;; as a subprocess and asserts exit 0 plus `a-control=Ok`, `a-boom   =Lost` and `b-after  =Ok` —
;; the innocent second client being served is the invariant, and the world it fences against
;; printed `b-dial=connect-REFUSED` from the connect arm below. ⛔ The LABEL TEXT AND ITS PADDING
;; ARE PART OF THE CONTRACT now (`"a-boom   "`, `"b-after  "`): a change here can redden the
;; floor. Run `cargo nextest run --release -E 'binary_id(wat::probes)'` after editing.
;;
;; This is the baseline the no-crash wall will flip. The handler here raises the SAME way the 19
;; `"redial failed — peer is dead"` arms do (`assertion-failed!` inside an `:impls` body), so what
;; this probe prints is what those 19 arms do to a live service today.
;;
;; Shape copied from `probe-arc278-wire-dos-service-killed.wat` (arc 278 stone 2), which measured
;; the same wall one cause narrower: a MALFORMED REQUEST can no longer crash a service. A handler
;; that raises on its own still can — that is the gap D1-a closes.
;;
;; ⛔ Every observation PRINTS rather than raising, including the connect arms: a probe that dies on
;; the thing it is measuring reports nothing. Non-vacuity control comes first (n=1 → Ok) so an
;; all-failures run cannot be read as a wall.
(:wat::core::defsurface :nc::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :nc::Svc::GoRequest [n <- :wat::core::i64])
   (:wat::core::defenum :nc::Svc::GoResponse :wat::enum::Pure
     :Ok               [got <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(go [self <- :nc::Svc  req <- :nc::Svc::GoRequest] -> :nc::Svc::GoResponse :max-request-bytes 4096)])

(:wat::service::defservice :nc::svc
  :satisfies :nc::Svc
  :durable   [hits <- :wat::core::i64]
  :ephemeral []
  :impls
  [(go [s ctx req]
     ;; n = 0 → the handler raises, exactly as the redial-fail arms do.
     (:wat::core::if (:wat::i64::= (:nc::Svc::GoRequest/n req) 0)
       (:wat::kernel::assertion-failed!
         "nc: the handler raised — the 19-arm shape (redial failed — peer is dead)"
         :wat::core::None :wat::core::None)
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:nc::Svc::Reply::Go (:nc::Svc::GoResponse::Ok (:nc::Svc::GoRequest/n req))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:nc::Svc::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:nc::svc::Op])]))))])

(:wat::core::defn :nc::try
  [c <- (:wat::kernel::Peer :- [:nc::Svc::Op :nc::Svc::Reply])  label <- :wat::core::String
   n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match (:nc::Svc/go c (:nc::Svc::GoRequest :n n))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:nc::Svc::GoResponse::Ok got)
          (:wat::kernel::println (:wat::string::concat label "=Ok")))
        ((:nc::Svc::GoResponse::RequestTooLarge b cap)
          (:wat::kernel::println (:wat::string::concat label "=RequestTooLarge")))
        ((:nc::Svc::GoResponse::RequestMalformed p e g)
          (:wat::kernel::println (:wat::string::concat label "=RequestMalformed")))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::string::concat label "=Lost")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println (:wat::string::concat label "=Stopped")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::string::concat label "=Closed")))
    (:wat::kernel::RecvOutcome::TimedOut
      (:wat::kernel::println (:wat::string::concat label "=TimedOut")))
    ((:wat::kernel::RecvOutcome::Malformed cause)
      (:wat::kernel::println (:wat::string::concat label "=Malformed")))))

;; Connect and REPORT — never raise. A refused connect is this probe's headline observation.
(:wat::core::defn :nc::dial
  [h <- :nc::svc::Handle  label <- :wat::core::String]
  -> (:wat::core::Option :- [(:wat::kernel::Peer :- [:nc::Svc::Op :nc::Svc::Reply])])
  (:wat::core::match (:wat::kernel::connect (:nc::svc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) (:wat::core::Some p))
    ((:wat::kernel::ConnectOutcome::Refused f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-REFUSED"))] :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-REJECTED"))] :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-FAILED"))] :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:nc::svc/start :locus (:wat::spawn::process) :record (:nc::svc::Record :hits 0))
     ;; client A
     _ (:wat::core::match (:nc::dial h "a-dial")
         ((:wat::core::Some a)
           (:wat::core::let
             [_ (:nc::try a "a-control" 1)     ;; NON-VACUITY: the service answers
              _ (:nc::try a "a-boom   " 0)]    ;; the handler raises
             nil))
         (:wat::core::None nil))
     ;; client B connects AFTER the raise — an INNOCENT second client
     _ (:wat::core::match (:nc::dial h "b-dial")
         ((:wat::core::Some b) (:nc::try b "b-after  " 1))
         (:wat::core::None nil))]
    nil))
