;; probe-redial-from-durable-addr.wat — can a service reacquire its own connection?
;;
;; Every service in the circuit takes an Address as an :init parameter, dials it once, keeps
;; only the Peer, and DISCARDS THE ADDRESS. So when a pipe breaks there is nothing to re-dial:
;; all 20 RecvOutcome::Lost arms across circuit/topic/queue are `assertion-failed!`. A lost
;; connection is fatal because recovery was never expressible.
;;
;; The fix follows the substrate's own doctrine -- :durable is "the soul: EDN, crosses the wire,
;; survives hibernation"; :ephemeral is "the body: resources and peer clients, never crosses".
;; An Address IS a soul. A Peer IS a body. We have been keeping the body and throwing the soul away.
;;
;; NO EXEMPLAR EXISTS: nothing in wat/ or wat-scripts/ holds an Address in :durable. This would
;; be the first. So three things are unproven and all three are asked here:
;;
;;   1. can :durable hold an (Address :- [Op Reply]) at all?
;;   2. can an ARM re-dial from it and put the new Peer in returned state -- or does the
;;      excursus-002 handle-lifetime wall reject a Peer created outside :init?
;;   3. does the re-dialed Peer actually work?

(:wat::core::defsurface :rd::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :rd::Echo::PingRequest [])
   (:wat::core::defenum :rd::Echo::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :rd::Echo  req <- :rd::Echo::PingRequest]
     -> :rd::Echo::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :rd::echo
  :satisfies :rd::Echo
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:rd::Echo::Reply::Ping (:rd::Echo::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:rd::Echo::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:rd::echo::Op])])))])

(:wat::core::defsurface :rd::Dialer :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :rd::Dialer::HitRequest [])
   (:wat::core::defenum :rd::Dialer::HitResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :rd::Dialer::RedialRequest [])
   (:wat::core::defenum :rd::Dialer::RedialResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(hit    [self <- :rd::Dialer  req <- :rd::Dialer::HitRequest]
     -> :rd::Dialer::HitResponse :max-request-bytes 524288)
   (redial [self <- :rd::Dialer  req <- :rd::Dialer::RedialRequest]
     -> :rd::Dialer::RedialResponse :max-request-bytes 524288)])

;; ★ THE QUESTION: an Address in :durable, and a Peer in :ephemeral, per the doctrine.
(:wat::service::defservice :rd::dialer
  :satisfies :rd::Dialer
  :durable   [target <- (:wat::kernel::Address :- [:rd::Echo::Op :rd::Echo::Reply])]
  :ephemeral [peer <- (:wat::kernel::Peer :- [:rd::Echo::Op :rd::Echo::Reply])]
  :peers     [:rd::Echo]
  :init (:wat::core::fn [record <- :rd::dialer::Record] -> :rd::dialer::State
          (:rd::dialer::State :durable record
            :peer (:wat::core::match (:wat::kernel::connect (:rd::dialer::Record/target record))
                    ((:wat::kernel::ConnectOutcome::Connected p) p)
                    (_ (:wat::kernel::assertion-failed! "rd: init dial failed" :wat::core::None :wat::core::None)))))
  :impls
  [(hit [s ctx req]
     (:wat::core::let
       [ok (:wat::core::match (:rd::Echo/ping (:rd::dialer::State/peer s) (:rd::Echo::PingRequest))
             ((:wat::kernel::RecvOutcome::Message _r) true)
             (_ false))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:rd::Dialer::Reply::Hit (:rd::Dialer::HitResponse::Ok ok)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:rd::Dialer::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:rd::dialer::Op])]))))

   ;; ★ RE-DIAL FROM THE DURABLE ADDRESS, INSIDE AN ARM, and keep the new Peer in state.
   ;; This is the shape excursus 002's handle-lifetime wall might reject.
   (redial [s ctx req]
     (:wat::core::let
       [addr (:rd::dialer::Record/target (:rd::dialer::State/durable s))
        fresh (:wat::core::match (:wat::kernel::connect addr)
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                (_ (:wat::kernel::assertion-failed! "rd: redial failed" :wat::core::None :wat::core::None)))
        s' (:rd::dialer::State :durable (:rd::dialer::State/durable s) :peer fresh)]
       (:wat::service::Outcome::Continue s'
         (:wat::core::Some (:rd::Dialer::Reply::Redial (:rd::Dialer::RedialResponse::Ok true)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:rd::Dialer::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:rd::dialer::Op])]))))])

(:wat::core::defn :rd::dial-dialer
  [a <- (:wat::kernel::Address :- [:rd::Dialer::Op :rd::Dialer::Reply])] -> :rd::Dialer
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "rd: dial-dialer failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :rd::hit! [c <- :rd::Dialer] -> :wat::core::String
  (:wat::core::match (:rd::Dialer/hit c (:rd::Dialer::HitRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:rd::Dialer::HitResponse::Ok ok) (:wat::core::if ok "yes" "no"))
        (_ "other")))
    (_ "lost")))

(:wat::core::defn :rd::redial! [c <- :rd::Dialer] -> :wat::core::String
  (:wat::core::match (:rd::Dialer/redial c (:rd::Dialer::RedialRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:rd::Dialer::RedialResponse::Ok ok) (:wat::core::if ok "yes" "no"))
        (_ "other")))
    (_ "lost")))

(:wat::core::defn :rd::run [] -> :wat::core::String
  (:wat::core::let
    [eh (:rd::echo/start :locus (:wat::spawn::thread) :record (:rd::echo::Record))
     dh (:rd::dialer/start :locus (:wat::spawn::thread)
          :record (:rd::dialer::Record :target (:rd::echo::Handle/addr eh)))
     c  (:rd::dial-dialer (:rd::dialer::Handle/addr dh))
     before (:rd::hit! c)
     red    (:rd::redial! c)
     after  (:rd::hit! c)
     out (:wat::core::format "durable-addr=ok;before={b};redial={r};after={a}"
           :b before :r red :a after)]
    out))

(:wat::core::defn :user::compute [] -> :wat::core::String (:rd::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:rd::run)))
