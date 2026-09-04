
;; probe-none-spellings-diverge.wat — ⛔ ITS FIRST HEADER OVERCLAIMED. CORRECTED.
;;
;; WHAT THIS IS NOT: an Option defect. Option/Some/None are fine and every other user of
;; them is fine, because every other user puts Option in an ORDINARY FUNCTION'S RETURN
;; POSITION. This probe puts it in the `reply` field of `Outcome::Continue` — a serve-loop
;; protocol slot. Whatever happens here is about the generated serve loop, not Option.
;;
;; ON THE FORM: `(:wat::core::None <Type>)` as a CONSTRUCTOR is real but nearly unique —
;; ONE site in the whole corpus, positional-to-kwargs.wat:27, in a plain function returning
;; (Option :- [WatAST]). The ~150 other `(:wat::core::None X)` occurrences are MATCH ARMS
;; (pattern, then the arm's body), not constructor calls. The first header's framing of
;; "two spellings of a value" was drawn from that miscount.
;;
;; WHAT IS ACTUALLY MEASURED:
;;   (:wat::core::None :cd::Drop::Reply)   caller LOST ; served-after=-1 ; redial=REFUSED
;;                                         the service is GONE                     (3/3)
;;   :wat::core::None                      the caller never returns                (exit 124)
;;                                         ⛔ SERVICE STATE UNMEASURED — the hang prevents
;;                                            this file's own liveness check from running.
;;
;; ⛔ THOSE TWO ROWS ARE NOT A COMPARISON. One is a measured death; the other is a hang with
;; the liveness question unanswered. The first header set them side by side as if both had
;; been observed.
;;
;; AND THE HANG IS NOT A DEFECT. `None` in the reply slot means "I am not replying now."
;; sqs.wat:584-585 uses exactly that for the queue's long-poll park and works, because the
;; TICK ANSWERS LATER. This probe never answers, so its caller waits forever. That is the
;; deferred-reply contract behaving as specified.
;;
;; ★ THE ONE REAL OPEN QUESTION: why does the typed constructor in the reply slot take the
;; service down? One call site, unexplained, and NOT established to be about the spelling.

(:wat::config::set-redef! true)

(:wat::core::defsurface :cd::Drop :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :cd::Drop::HitRequest [tag <- :wat::core::i64])
   (:wat::core::defenum :cd::Drop::HitResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :cd::Drop::ServedRequest [])
   (:wat::core::defenum :cd::Drop::ServedResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(hit    [self <- :cd::Drop  req <- :cd::Drop::HitRequest]    -> :cd::Drop::HitResponse :max-request-bytes 65536)
   (served [self <- :cd::Drop  req <- :cd::Drop::ServedRequest] -> :cd::Drop::ServedResponse :max-request-bytes 65536)])

(:wat::service::defservice :cd::drop
  :satisfies :cd::Drop
  :durable   [served <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :cd::drop::Record] -> :cd::drop::State
          (:cd::drop::State :durable record))
  :impls
  [(hit [s ctx req]
     (:wat::core::let
       [rec (:cd::drop::State/durable s)
        n   (:wat::i64::+ (:cd::drop::Record/served rec) 1)
        s'  (:cd::drop::State :durable (:cd::drop::Record :served n))
        no-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:cd::Drop::Reply])])
        no-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:cd::drop::Op])])]
       ;; ★ THE DROP. The arm RAN and the state advanced either way; only the reply differs.
       (:wat::core::if (:wat::core::= n 2)
         (:wat::service::Outcome::Continue s'
           :wat::core::None                             ;; UNTYPED — the only variable
           no-sends no-arms)
         (:wat::service::Outcome::Continue s'
           (:wat::core::Some (:cd::Drop::Reply::Hit (:cd::Drop::HitResponse::Ok n)))
           no-sends no-arms))))
   (served [s ctx req]
     (:wat::core::let
       [rec (:cd::drop::State/durable s)
        no-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:cd::Drop::Reply])])
        no-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:cd::drop::Op])])]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:cd::Drop::Reply::Served
           (:cd::Drop::ServedResponse::Ok (:cd::drop::Record/served rec))))
         no-sends no-arms)))])

(:wat::core::defn :cd::hit [p <- :cd::Drop  tag <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match (:cd::Drop/hit p (:cd::Drop::HitRequest :tag tag))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:cd::Drop::HitResponse::Ok n) (:wat::core::format "ok:{n}" :n n))
        ((:cd::Drop::HitResponse::RequestTooLarge _b _c) "TOO-LARGE")
        ((:cd::Drop::HitResponse::RequestMalformed _p _e _g) "MALFORMED")))
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :cd::served-count [p <- :cd::Drop] -> :wat::core::i64
  (:wat::core::match (:cd::Drop/served p (:cd::Drop::ServedRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:cd::Drop::ServedResponse::Ok n) n)
        (_ -1)))
    (_ -1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:cd::drop/start :locus (:wat::spawn::process) :record (:cd::drop::Record :served 0))
     p (:wat::core::match (:wat::kernel::connect (:cd::drop::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "cd: dial failed" :wat::core::None :wat::core::None)))
     a (:cd::hit p 1)
     _ (:wat::kernel::println (:wat::core::format "call1={a}" :a a))
     ;; call 2 is the dropper — the arm runs, the state advances, no reply is sent.
     b (:cd::hit p 2)
     _ (:wat::kernel::println (:wat::core::format "call2-RETURNED={b}" :b b))
     ;; ★ THE QUESTION I DID NOT ASK. `served-count` was defined at :89 and never called.
     ;; LOST tells you the caller got nothing. It does NOT tell you WHY. Two worlds print
     ;; the same string: "a reply was omitted on a living connection", and "the service
     ;; died". Only a liveness check separates them — and the instrument was already here.
     alive (:cd::served-count p)
     redial (:wat::core::match (:wat::kernel::connect (:cd::drop::Handle/addr h))
              ((:wat::kernel::ConnectOutcome::Connected _c) "reconnected")
              (_ "REFUSED"))]
    (:wat::kernel::println
      (:wat::core::format "served-after={s};redial={r};verdict={v}"
        :s alive :r redial
        :v (:wat::core::if (:wat::i64::>= alive 0)
             "reply-omitted-service-LIVES" "service-DIED-the-LOST-was-death")))))
