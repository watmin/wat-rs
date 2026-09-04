;; probe-a-none-reply-is-a-promise.wat — WHAT `None` IN THE REPLY SLOT ACTUALLY MEANS.
;;
;; The builder asked what I was attempting in that return position. I was attempting
;; "advance the state and send the caller nothing", and I wrote it two ways, both wrong:
;;
;;   (:wat::core::None :cd::Drop::Reply)   a PHANTOM FORM -- None is a keyword, not a
;;                                         callable. Type-checks, raises UnknownFunction,
;;                                         kills the service. See arc 109's
;;                                         NOTE-none-is-not-a-function.md.
;;   :wat::core::None  (and never answer)  the correct spelling, but an INCOHERENT program:
;;                                         None does not mean "the caller gets nothing".
;;
;; ★ `None` IN THE REPLY SLOT IS A PROMISE TO ANSWER LATER. Not a refusal. The caller stays
;; parked on its recv until something sends to its conn-id. `sqs.wat:584-585` relies on
;; exactly this for the queue's long-poll park, and answers from the tick.
;;
;; This is the minimal exemplar of that contract, which the tree did not have -- sqs's park
;; is embedded in a 900-line queue. Two arms and a timer:
;;
;;   ask     -> stash ctx's conn-id, reply None, arm a 60 ms timer      (the promise)
;;   -settle -> SelfOutcome::Continue with a Directed to that conn-id   (the promise KEPT)
;;
;; ⛔ Addressed by conn-id, NEVER by peer: wat/service.wat:60-62 -- "an arm must not hold a
;; caller's Peer -- :durable crosses the wire, :ephemeral is the body, neither is an honest
;; home. The arm names; the serve loop resolves against selectables and sends."
;;
;; expect: answered=ok:42;waited-ms>=60  — the caller blocked, then was answered.

(:wat::config::set-redef! true)

(:wat::core::defsurface :pr::Late :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pr::Late::AskRequest [n <- :wat::core::i64])
   (:wat::core::defenum :pr::Late::AskResponse :wat::enum::Pure
     :Ok [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ask [self <- :pr::Late  req <- :pr::Late::AskRequest]
     -> :pr::Late::AskResponse :max-request-bytes 65536)])

(:wat::service::defservice :pr::late
  :satisfies :pr::Late
  :durable   [waiter <- :wat::core::i64  answer <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :pr::late::Record] -> :pr::late::State
          (:pr::late::State :durable record))
  :impls
  [(ask [s ctx req]
     ;; THE PROMISE. Stash who is waiting, reply None, arm the settle.
     (:wat::service::Outcome::Continue
       (:pr::late::State :durable
         (:pr::late::Record
           :waiter (:wat::service::Invocation/conn-id ctx)
           :answer (:pr::Late::AskRequest/n req)))
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:pr::Late::Reply])])
       [(:wat::service::Alarm :after (:wat::time::Millisecond 60) :op :-settle)]))
   (-settle [s ctx]
     ;; THE PROMISE KEPT. Internal arm: SelfOutcome has no reply field — it cannot answer
     ;; an invoker, because it has none. It answers the STASHED conn-id via `sends`.
     (:wat::core::let
       [rec (:pr::late::State/durable s)]
       (:wat::service::SelfOutcome::Continue s
         (:wat::core::Vector :- [(:wat::service::Directed :- [:pr::Late::Reply])]
           (:wat::service::Directed
             :conn-id (:pr::late::Record/waiter rec)
             :reply (:pr::Late::Reply::Ask
                      (:pr::Late::AskResponse::Ok (:pr::late::Record/answer rec)))))
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:pr::late::Op])]))))])

(:wat::core::defn :pr::ask [p <- :pr::Late  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match (:pr::Late/ask p (:pr::Late::AskRequest :n n))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:pr::Late::AskResponse::Ok k) (:wat::core::format "ok:{k}" :k k))
        ((:pr::Late::AskResponse::RequestTooLarge _b _c) "TOO-LARGE")
        ((:pr::Late::AskResponse::RequestMalformed _p _e _g) "MALFORMED")))
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:pr::late/start :locus (:wat::spawn::process)
         :record (:pr::late::Record :waiter -1 :answer 0))
     p (:wat::core::match (:wat::kernel::connect (:pr::late::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pr: dial failed" :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     a  (:pr::ask p 42)
     t1 (:wat::time::epoch-nanos (:wat::time::now))
     ms (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    (:wat::kernel::println
      (:wat::core::format "answered={a};waited-ms={ms};verdict={v}"
        :a a :ms ms
        :v (:wat::core::if
             (:wat::core::and (:wat::core::= a "ok:42") (:wat::i64::>= ms 55))
             "NONE-IS-A-PROMISE-AND-IT-WAS-KEPT" "see-cells")))))
