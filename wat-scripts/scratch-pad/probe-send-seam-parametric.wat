;; probe-send-seam-parametric.wat — THE PROBE R1's FIRST DRAFT SHIPPED WITHOUT.
;;
;; R1 was struck down on STOP-1: I gave the seam `Peer :- [Never R]`, which is the TIMER
;; orientation (service.wat: "after is honestly typed (Peer :- [Never O]) — it can never
;; RECEIVE a Reply"). `send` projects I from `Peer :- [I O]`, so a helper that sends :R
;; needs `Peer :- [R O]` — TWO type parameters.
;;
;; ★ THE QUESTION: does a parametric top-level defn over TWO params accept a peer and a
;; payload of that peer's INPUT type, and does `:wat::kernel::send` type-check inside it?
;;
;; Precedent for the form: 50 parametric defns exist, incl. `:wat::core::foldl-spec :- [T U]`
;; (wat/seq.wat:277) — two params. So the SHAPE is not in question; the peer/payload
;; projection is.
;;
;; ⚠ WHAT THIS PROBE CAN AND CANNOT SEE. It calls the helper with a CLIENT peer
;; (Peer :- [Op Reply]) sending an Op, so R=Op here. The serve loop's peer is
;; (Peer :- [Reply Op]) sending a Reply, so R=Reply there. Same constraint — the payload
;; must be the peer's I — and the same signature. It proves the SIGNATURE, not the
;; template's call sites.
;;
;;   expect: sent=yes;verdict=SEAM-EXPRESSES

(:wat::config::set-redef! true)

(:wat::core::defsurface :ss::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :ss::Echo::PingRequest [])
   (:wat::core::defenum :ss::Echo::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :ss::Echo  req <- :ss::Echo::PingRequest]
     -> :ss::Echo::PingResponse :max-request-bytes 65536)])

(:wat::service::defservice :ss::echo
  :satisfies :ss::Echo
  :durable   [tag <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :ss::echo::Record] -> :ss::echo::State
          (:ss::echo::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:ss::Echo::Reply::Ping (:ss::Echo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:ss::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:ss::echo::Op])])))])

;; ★ THE SEAM, with the orientation R1's draft got wrong. Body is service.wat's four arms
;; verbatim: Sent/Closed keep serving, Stopped is the world stopping, Lost keeps serving.
(:wat::core::defn :ss::send-keep-serving? :- [R O]
  [peer <- (:wat::kernel::Peer :- [:R :O])  payload <- :R] -> :wat::core::bool
  (:wat::core::match (:wat::kernel::send peer payload)
    (:wat::kernel::SendOutcome::Sent   true)
    (:wat::kernel::SendOutcome::Closed true)
    (:wat::kernel::SendOutcome::Stopped false)
    ((:wat::kernel::SendOutcome::Lost _c) true)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:ss::echo/start :locus (:wat::spawn::thread) :record (:ss::echo::Record :tag 1))
     p (:wat::core::match (:wat::kernel::connect (:ss::echo::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "ss: dial failed" :wat::core::None :wat::core::None)))
     ok (:ss::send-keep-serving? p (:ss::Echo::Op::Ping (:ss::Echo::PingRequest)))]
    (:wat::kernel::println
      (:wat::core::format "sent={s};verdict={v}"
        :s (:wat::core::if ok "yes" "no")
        :v (:wat::core::if ok "SEAM-EXPRESSES" "SEAM-DOES-NOT")))))
