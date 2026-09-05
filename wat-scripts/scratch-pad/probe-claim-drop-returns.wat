;; Does a None reply on Seen-like claim return LOST to a process client, or block?
;; ⛔ 3d ARTIFACT — its premise was REFUTED, and it carried the phantom form.
;;
;; Built to test whether a service arm could omit a reply and leave the caller informed
;; while the service lived. There is no such outcome: a reply is SENT or it is DEFERRED.
;;
;; It also used `(:wat::core::None <Type>)`, which is not a form -- None is a KEYWORD, not
;; a callable. That spelling type-checks for a non-primitive type keyword and raises
;; UnknownFunction at runtime, which is what killed the service and was misread as
;; "the reply was omitted". Corrected below to the only valid spelling, per the builder's
;; ruling for this branch: avoid the flaw.
;;
;; See docs/arc/2026/04/109-kill-std/NOTE-none-is-not-a-function.md and
;; wat-scripts/scratch-pad/probe-none-is-not-a-function.wat.
;;
;; ⚠ With the correct spelling the arm DEFERS, so a caller that is never answered waits
;; forever. Run under `timeout`. That is the contract, not a defect.

(:wat::config::set-redef! true)

(:wat::core::defsurface :pd::S :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pd::S::ClaimRequest [seq <- :wat::core::String])
   (:wat::core::defenum :pd::S::ClaimResponse :wat::enum::Pure
     :First []
     :Dup []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :pd::S  req <- :pd::S::ClaimRequest] -> :pd::S::ClaimResponse :max-request-bytes 65536)])

(:wat::service::defservice :pd::s
  :satisfies :pd::S
  :durable   []
  :ephemeral []
  :init (:wat::core::fn [record <- :pd::s::Record] -> :pd::s::State
          (:pd::s::State :durable record))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [sends (:wat::core::Vector :- [(:wat::service::Directed :- [:pd::S::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:pd::s::Op])])]
       (:wat::service::Outcome::Continue s
         :wat::core::None
         sends none-alarms)))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:pd::s/start :locus (:wat::spawn::process) :record (:pd::s::Record))
     p (:wat::core::match (:wat::kernel::connect (:pd::s::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pd: dial failed" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println "calling")
     r (:wat::core::match (:pd::S/claim p (:pd::S::ClaimRequest :seq "x"))
         ((:wat::kernel::RecvOutcome::Message _) "MESSAGE")
         ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
         (:wat::kernel::RecvOutcome::Stopped "STOPPED")
         (:wat::kernel::RecvOutcome::Closed "CLOSED") (:wat::kernel::RecvOutcome::TimedOut "LOST"))
     live (:wat::core::match (:wat::kernel::connect (:pd::s::Handle/addr h))
             ((:wat::kernel::ConnectOutcome::Connected _) "ALIVE")
             (_ "DEAD"))]
    (:wat::kernel::println (:wat::core::format "claim={c};svc={s}" :c r :s live))))
