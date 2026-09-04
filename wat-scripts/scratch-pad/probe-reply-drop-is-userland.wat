;; probe-reply-drop-is-userland.wat — CAN 3d BE BUILT WITHOUT REACTOR SURGERY?
;;
;; wat/service.wat is 3120 lines containing exactly ONE top-level form: the defservice
;; macro. There is no helper function — the whole serve loop is generated inside one
;; quasiquote, and every service in the corpus expands through it. Weaving a drop in there
;; is surgery with total blast radius.
;;
;; ★ BUT `Outcome::Continue` carries `reply <- (Option :- [R])`. An arm can do its work,
;; emit its sends, advance its state, and return NO REPLY. That is precisely the 3d fault
;; — "after the arm, before the reply-send: work happened, caller does not know" — and if
;; it behaves as the type suggests, 3d is a USERLAND stone.
;;
;; The drop here is by call COUNT, not random: rand is already proven replayable
;; (probe-rand-is-usable-from-wat.wat) and mixing the two would confound this question.
;; Call 1 replies. Call 2 returns None.
;;
;; SELF-GUARDING BY ORDERING: the answering call runs FIRST and prints. The dropped call
;; runs LAST, so the shell's `timeout` is the watchdog and the printed lines are the
;; evidence. No wait in this file can swallow its own result.
;;
;;   $ timeout 20 ./target/release/wat .../probe-reply-drop-is-userland.wat
;;   call1=ok:1          <- normal reply
;;   served=2            <- the ARM RAN on call 2 (state advanced) …
;;   (then: does call2 return, or does the caller wait forever?)

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
           (:wat::core::None :cd::Drop::Reply)          ;; work happened, caller told nothing
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
