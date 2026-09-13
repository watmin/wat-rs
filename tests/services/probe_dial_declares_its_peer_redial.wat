;; the-dial-declares-its-peer — the declared redial still compiles and runs.
;; Same shape as the refusal, plus `:peers [:probe::Mid]` and an `:ephemeral`
;; `Peer<Mid>`. The handler redials `Record/mid-addr` (the worker's `-disrupt`
;; shape). Expect "pong:hi".

(:wat::core::defsurface :probe::Mid :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Mid::PingRequest  [msg <- :wat::core::String])
   (:wat::core::defenum :probe::Mid::PingResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Mid  req <- :probe::Mid::PingRequest] -> :probe::Mid::PingResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::mid
  :satisfies :probe::Mid
  :durable   []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Mid::Reply::Ping (:probe::Mid::PingResponse::Ok
         (:wat::string::concat "pong:" (:probe::Mid::PingRequest/msg req)))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Mid::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::mid::Op])])))])

(:wat::core::defsurface :probe::Front :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Front::RunRequest  [])
   (:wat::core::defenum :probe::Front::RunResponse :wat::enum::Pure
     :Ok              [out <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(run [self <- :probe::Front  req <- :probe::Front::RunRequest] -> :probe::Front::RunResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::front
  :satisfies :probe::Front
  :durable   [mid-addr <- (:wat::kernel::Address :- [:probe::Mid::Op :probe::Mid::Reply])]
  :ephemeral [mid <- (:wat::kernel::Peer :- [:probe::Mid::Op :probe::Mid::Reply])]
  :peers     [:probe::Mid]
  :init (:wat::core::fn
          [record <- :probe::front::Record]
          -> :probe::front::State
          (:probe::front::State :durable record
            :mid (:wat::core::match
                   (:wat::kernel::connect (:probe::front::Record/mid-addr record))
                   ((:wat::kernel::ConnectOutcome::Connected p) p)
                   ((:wat::kernel::ConnectOutcome::Refused c)
                     (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                   ((:wat::kernel::ConnectOutcome::Rejected c)
                     (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                   ((:wat::kernel::ConnectOutcome::Failed c)
                     (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))))
  :impls
  [(run [s ctx req]
     (:wat::core::let
       [fresh (:wat::core::match
                (:wat::kernel::connect (:probe::front::Record/mid-addr (:probe::front::State/durable s)))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                (_ (:wat::kernel::assertion-failed! "front: redial failed — peer is dead, not a broken pipe" :wat::core::None :wat::core::None)))
        er    (:probe::Mid/ping fresh (:probe::Mid::PingRequest :msg "hi"))
        rresp (:wat::core::match er
                ((:wat::kernel::RecvOutcome::Message __recv)
                  (:wat::core::match __recv
                    ((:probe::Mid::PingResponse::Ok reply)
                      (:probe::Front::RunResponse::Ok reply))
                    ((:probe::Mid::PingResponse::RequestTooLarge bytes cap)
                      (:probe::Front::RunResponse::RequestTooLarge bytes cap))
                    ((:probe::Mid::PingResponse::RequestMalformed mpath mexpected mgot)
                      (:probe::Front::RunResponse::RequestMalformed mpath mexpected mgot))))
                ((:wat::kernel::RecvOutcome::Lost __cause)
                  (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Stopped
                  (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::Closed
                  (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))
                (:wat::kernel::RecvOutcome::TimedOut
                  (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))
                ((:wat::kernel::RecvOutcome::Malformed _cause)
                  (:wat::kernel::assertion-failed! "recv: malformed frame" :wat::core::None :wat::core::None)))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:probe::Front::Reply::Run rresp))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Front::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::front::Op])]))))])

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [mh (:probe::mid/start  :locus (:wat::spawn::thread) :record (:probe::mid::Record))
     ma (:probe::mid::Handle/addr mh)
     fh (:probe::front/start :locus (:wat::spawn::thread)
           :record (:probe::front::Record :mid-addr ma))
     fc (:wat::core::match (:wat::kernel::connect (:probe::front::Handle/addr fh))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused c)
            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected c)
            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed c)
            (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     rr (:probe::Front/run fc (:probe::Front::RunRequest))]
    (:wat::core::match rr
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:probe::Front::RunResponse::Ok out) out)
          ((:probe::Front::RunResponse::RequestTooLarge bytes cap)
            (:wat::kernel::assertion-failed! "compute: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::Front::RunResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::TimedOut
        (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Malformed _cause)
        (:wat::kernel::assertion-failed! "recv: malformed frame" :wat::core::None :wat::core::None)))))
