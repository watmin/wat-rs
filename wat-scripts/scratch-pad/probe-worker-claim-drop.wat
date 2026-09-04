;; A defservice worker claiming a dropping seen — the circuit topology.
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

(:wat::core::defsurface :pw::S :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pw::S::ClaimRequest [seq <- :wat::core::String])
   (:wat::core::defenum :pw::S::ClaimResponse :wat::enum::Pure
     :First []
     :Dup []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(claim [self <- :pw::S  req <- :pw::S::ClaimRequest] -> :pw::S::ClaimResponse :max-request-bytes 65536)])

(:wat::service::defservice :pw::s
  :satisfies :pw::S
  :durable   []
  :ephemeral []
  :init (:wat::core::fn [record <- :pw::s::Record] -> :pw::s::State
          (:pw::s::State :durable record))
  :impls
  [(claim [s ctx req]
     (:wat::core::let
       [sends (:wat::core::Vector :- [(:wat::service::Directed :- [:pw::S::Reply])])
        none-alarms (:wat::core::Vector :- [(:wat::service::Alarm :- [:pw::s::Op])])]
       (:wat::service::Outcome::Continue s
         :wat::core::None
         sends none-alarms)))])

(:wat::core::defsurface :pw::W :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pw::W::StartRequest [])
   (:wat::core::defenum :pw::W::StartResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :pw::W::GotRequest [])
   (:wat::core::defenum :pw::W::GotResponse :wat::enum::Pure
     :Ok [r <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :pw::W  req <- :pw::W::StartRequest] -> :pw::W::StartResponse :max-request-bytes 65536)
   (got [self <- :pw::W  req <- :pw::W::GotRequest] -> :pw::W::GotResponse :max-request-bytes 65536)])

(:wat::service::defservice :pw::w
  :satisfies :pw::W
  :durable   [s-addr <- (:wat::kernel::Address :- [:pw::S::Op :pw::S::Reply])]
  :ephemeral [s <- (:wat::kernel::Peer :- [:pw::S::Op :pw::S::Reply])
              got <- :wat::core::String]
  :peers     [:pw::S]
  :init (:wat::core::fn [record <- :pw::w::Record] -> :pw::w::State
          (:pw::w::State :durable record
            :s (:wat::core::match (:wat::kernel::connect (:pw::w::Record/s-addr record))
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 (_ (:wat::kernel::assertion-failed! "pw: dial s failed" :wat::core::None :wat::core::None)))
            :got "pending"))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:pw::W::Reply::Start (:pw::W::StartResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:pw::W::Reply])])
       [(:wat::service::Alarm :delay (:wat::time::Milliseconds 10) :op :-tick)]))
   (got [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:pw::W::Reply::Got (:pw::W::GotResponse::Ok (:pw::w::State/got s))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:pw::W::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:pw::w::Op])])))
   (-tick [s ctx]
     (:wat::core::let
       [r (:wat::core::match (:pw::S/claim (:pw::w::State/s s) (:pw::S::ClaimRequest :seq "x"))
            ((:wat::kernel::RecvOutcome::Message _) "MESSAGE")
            ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
            (:wat::kernel::RecvOutcome::Stopped "STOPPED")
            (:wat::kernel::RecvOutcome::Closed "CLOSED"))
        s' (:pw::w::State :durable (:pw::w::State/durable s)
             :s (:pw::w::State/s s) :got r)]
       (:wat::service::SelfOutcome::Continue s'
         (:wat::core::Vector :- [(:wat::service::Directed :- [:pw::W::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:pw::w::Op])]))))])

(:wat::core::defn :pw::pids [pl <- :wat::spawn::ProcessLaunch] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :pw::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    (_ nil)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sh (:pw::s/start :locus (:wat::spawn::process) :record (:pw::s::Record))
     wh (:pw::w/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:pw::s/grant sh (:pw::pids pl))))
          :record (:pw::w::Record :s-addr (:pw::s::Handle/addr sh)))
     w (:wat::core::match (:wat::kernel::connect (:pw::w::Handle/addr wh))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pw: dial w failed" :wat::core::None :wat::core::None)))
     _ (:wat::core::match (:pw::W/start w (:pw::W::StartRequest)) (_ nil))
     _ (:pw::nap 200)
     g (:wat::core::match (:pw::W/got w (:pw::W::GotRequest))
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::core::match r
             ((:pw::W::GotResponse::Ok s) s)
             (_ "BAD")))
         ((:wat::kernel::RecvOutcome::Lost _c) "GOT-LOST")
         (_ "GOT-OTHER"))]
    (:wat::kernel::println g)))
