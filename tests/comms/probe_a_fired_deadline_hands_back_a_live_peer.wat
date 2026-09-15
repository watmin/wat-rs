;; tests/comms/probe_a_fired_deadline_hands_back_a_live_peer.wat
;; STOP-1 (redials visible) and row 3 (dead peer → Lost, never DeadlineFired).
;; The two scratch-pad desync fixtures are the tag-flip acceptance tests.

(:wat::core::defsurface :live::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :live::Svc::GoRequest [tag <- :wat::core::i64])
   (:wat::core::defenum :live::Svc::GoResponse :wat::enum::Pure
     :Ok               [tag <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(go [self <- :live::Svc  req <- :live::Svc::GoRequest]
     -> :live::Svc::GoResponse :max-request-bytes 4096)])

(:wat::service::defservice :live::svc
  :satisfies :live::Svc
  :durable   [delay-ms <- :wat::core::i64]
  :ephemeral []
  :impls
  [(go [s ctx req]
     (:wat::core::let
       [_ (:wat::core::match
            (:wat::kernel::recv
              (:wat::kernel::after :wat::program::PeerKind::thread
                (:wat::time::Milliseconds (:live::svc::Record/delay-ms (:live::svc::State/durable s)))
                :done))
            ((:wat::kernel::RecvOutcome::Message _m) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil)
            (:wat::kernel::RecvOutcome::TimedOut nil)
            ((:wat::kernel::RecvOutcome::Malformed _c) nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:live::Svc::Reply::Go (:live::Svc::GoResponse::Ok (:live::Svc::GoRequest/tag req))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:live::Svc::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:live::svc::Op])]))))])

(:wat::core::defn :live::dial
  [h <- :live::svc::Handle]
  -> (:wat::kernel::Peer :- [:live::Svc::Op :live::Svc::Reply])
  (:wat::core::match (:wat::kernel::connect (:live::svc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused _c)
      (:wat::kernel::assertion-failed! "dial refused" :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected _c)
      (:wat::kernel::assertion-failed! "dial rejected" :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed _c)
      (:wat::kernel::assertion-failed! "dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :live::face
  [o <- (:wat::service::CallOutcome :- [:live::Svc::Reply])]
  -> :wat::core::String
  (:wat::core::match o
    ((:wat::service::CallOutcome::Answered reply)
      (:wat::core::match reply
        ((:live::Svc::Reply::Go resp)
          (:wat::core::match resp
            ((:live::Svc::GoResponse::Ok tag)
              (:wat::core::format "Answered:{t}" :t (:wat::i64::to-string tag)))
            (_ "Answered:other")))
        (_ "Answered:failed")))
    ((:wat::service::CallOutcome::Lost _c) "Lost")
    ((:wat::service::CallOutcome::Closed) "Closed")
    ((:wat::service::CallOutcome::DeadlineFired) "DeadlineFired")
    ((:wat::service::CallOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :user::redials-and-own-tag [] -> :wat::core::String
  (:wat::core::let
    [h (:live::svc/start :locus (:wat::spawn::process) :record (:live::svc::Record :delay-ms 600))
     c (:live::dial h)
     before (:wat::kernel::redials c)
     first (:live::face
             (:wat::service::call-by-deadline c
               (:live::Svc::Op::Go (:live::Svc::GoRequest :tag 11)) 100
               (:live::Svc::Reply::Go (:live::Svc::GoResponse::Ok -1))))
     after (:wat::kernel::redials c)
     second (:live::face
              (:wat::service::call-by-deadline c
                (:live::Svc::Op::Go (:live::Svc::GoRequest :tag 22)) 5000
                (:live::Svc::Reply::Go (:live::Svc::GoResponse::Ok -1))))]
    (:wat::core::format
      "before={b};first={f};after={a};second={s}"
      :b (:wat::i64::to-string before) :f first
      :a (:wat::i64::to-string after) :s second)))

(:wat::core::defn :user::dead-is-lost [] -> :wat::core::String
  (:wat::core::let
    [h (:live::svc/start :locus (:wat::spawn::process) :record (:live::svc::Record :delay-ms 600))
     c (:live::dial h)
     _ (:wat::service::stop-faced (:live::svc/stop h))]
    (:live::face
      (:wat::service::call-by-deadline c
        (:live::Svc::Op::Go (:live::Svc::GoRequest :tag 1)) 100
        (:live::Svc::Reply::Go (:live::Svc::GoResponse::Ok -1))))))
