;; probe-client-deadline-via-select.wat — CAN A CLIENT HAVE A REQUEST DEADLINE, TODAY?
;;
;; The builder's ruling: "clients must have a deadline on when they expect an answer by...
;; they discard the connection and try again if they don't get it."
;;
;; I proposed giving `recv` a deadline. That was wrong: it would be a SECOND timeout
;; implementation beside a working one. The serve loop already selects over timers and
;; connections together, at both tiers, with the crossbeam and io_uring work already
;; correct. A client should do what the loop does.
;;
;; ★ HOW THE LOOP UNIFIES A TIMER INTO ITS SELECTABLES — service.wat:1618-1623, and the
;; file names the trick itself: "the checker now reads it at (Peer :- [Reply O]) before it
;; ever reaches the Tuple ctor. Values are unaffected: Peer's type params are ERASED AT
;; RUNTIME — this is a check-time-only detour."
;;
;;     (:wat::core::first
;;       (:wat::core::conj (:wat::core::Vector :- [<selectable-peer-ty>])
;;         (:wat::kernel::after …)))
;;
;; The timer really is [nil O]; the vector's ANNOTATION retypes it. So the question this
;; probe answers is whether a CLIENT can perform the same laundering, with the orientation
;; inverted — the loop is [Reply Op], a client is [Op Reply].
;;
;; TWO CELLS, one variable — whether the server ever answers:
;;   fast   the server replies    -> select returns the REPLY      (deadline unused)
;;   never  the server defers and never settles -> select returns the TIMER
;;
;; The second cell is a server that has gone silent. Today that hangs a client forever;
;; with a deadline it is a survivable timeout.
;;
;;   expect: fast=reply;never=TIMED-OUT;verdict=CLIENT-DEADLINES-ARE-EXPRESSIBLE

(:wat::config::set-redef! true)

(:wat::core::defsurface :cs::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :cs::Echo::FastRequest [])
   (:wat::core::defenum :cs::Echo::FastResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :cs::Echo::NeverRequest [])
   (:wat::core::defenum :cs::Echo::NeverResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(fast  [self <- :cs::Echo  req <- :cs::Echo::FastRequest]  -> :cs::Echo::FastResponse  :max-request-bytes 65536)
   (never [self <- :cs::Echo  req <- :cs::Echo::NeverRequest] -> :cs::Echo::NeverResponse :max-request-bytes 65536)])

(:wat::service::defservice :cs::echo
  :satisfies :cs::Echo
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :cs::echo::Record] -> :cs::echo::State
          (:cs::echo::State :durable record))
  :impls
  [(fast [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:cs::Echo::Reply::Fast (:cs::Echo::FastResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:cs::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:cs::echo::Op])])))
   ;; defers and NEVER settles — a server gone silent
   (never [s ctx req]
     (:wat::service::Outcome::Continue s
       :wat::core::None
       (:wat::core::Vector :- [(:wat::service::Directed :- [:cs::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:cs::echo::Op])])))])

;; ★ THE LAUNDERING, client orientation: a client peer is (Peer :- [Op Reply]).
(:wat::core::defn :cs::deadline-peer
  [ms <- :wat::core::i64]
  -> (:wat::kernel::Peer :- [:cs::Echo::Op :cs::Echo::Reply])
  (:wat::core::first
    (:wat::core::conj
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:cs::Echo::Op :cs::Echo::Reply])])
      ;; ★ TIER MUST MATCH THE CONNECTION. select refuses a mixed-tier set: "a non-socket
    ;; peer among socket peers is not a representable-good state". The service is
    ;; process-locus, so its connection is a SOCKET peer and the timer must be one too.
    ;; The serve loop never has to choose -- it builds its timer with
    ;; (:wat::program::Env/peer-kind (:wat::program::env)), so its tier matches by
    ;; construction. A CLIENT must pick the tier of the peer it is racing.
    (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Milliseconds ms)
        (:cs::Echo::Reply::Fast (:cs::Echo::FastResponse::Ok))))))

;; send, then wait on EITHER the reply or the deadline — whichever fires first
(:wat::core::defn :cs::ask-with-deadline
  [p <- (:wat::kernel::Peer :- [:cs::Echo::Op :cs::Echo::Reply])
   op <- :cs::Echo::Op  ms <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match (:wat::kernel::send p op)
    (:wat::kernel::SendOutcome::Sent
      (:wat::core::let
        [tmr (:cs::deadline-peer ms)]
        (:wat::core::match (:wat::kernel::select [p tmr])
          ((:wat::spawn::ServiceEvent::Message idx _m)
            (:wat::core::if (:wat::i64::= idx 0) "reply" "TIMED-OUT"))
          ((:wat::spawn::ServiceEvent::Closed _i) "closed")
          ((:wat::spawn::ServiceEvent::Lost _i _c) "lost")
          ;; ServiceEvent also carries Shutdown/Admin/Connection/Malformed/Rejected --
          ;; a real deadline-aware client face must name each; a probe says so and moves on.
          (_ "other"))))
    (_ "send-failed")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:cs::echo/start :locus (:wat::spawn::process) :record (:cs::echo::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:cs::echo::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "cs: dial failed" :wat::core::None :wat::core::None)))
     a (:cs::ask-with-deadline p (:cs::Echo::Op::Fast (:cs::Echo::FastRequest)) 500)
     b (:cs::ask-with-deadline p (:cs::Echo::Op::Never (:cs::Echo::NeverRequest)) 200)]
    (:wat::kernel::println
      (:wat::core::format "fast={a};never={b};verdict={v}"
        :a a :b b
        :v (:wat::core::if
             (:wat::core::and (:wat::core::= a "reply") (:wat::core::= b "TIMED-OUT"))
             "CLIENT-DEADLINES-ARE-EXPRESSIBLE" "see-cells")))))
