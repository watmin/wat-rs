;; arc 255 Stone 255.25 (C-b4) — MEASUREMENT: the process child main spells its transport.
;; A process-locus defservice (the 272.6b fixture, verbatim below) started and driven end to end.
;; The generated child `:user::main` self-peer is `(Status :- [:wat::kernel::Transport.Wire])`
;; (see 255-17a-child-main-transport.wat for the expansion). Prints 5 when the child freezes and
;; serves; before the markers were a `Pure` family, spelling `Wire` there was refused by the
;; child's self-peer purity wall (255.17a: 57 process-child tests red).
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest        [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome.Reply {:state s
       :reply (:my::Counter::GetResponse.Ok {:value (:my::counter::Record/count (:my::counter::State/durable s))})}))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome.Reply {:state (:my::counter::State :durable (:my::counter::Record :count c))
                                      :reply (:my::Counter::IncrementResponse.Ok {:value c})})))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::process) :record (:my::counter::Record :count 0))
     c  (:wat::core::match (:wat::kernel::connect (:my::counter::Handle/addr h)) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _  (:wat::core::match (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
          [:wat::kernel::RecvOutcome.Message {:msg _resp} nil]
          [:wat::kernel::RecvOutcome.Lost {:cause _c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message _c))]
          [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE")]
          [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
     r  (:my::Counter/get c (:my::Counter::GetRequest))]
    (:wat::core::match r [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv
      [:my::Counter::GetResponse.Ok {:value value} value]
      ;; terminal test caller: an unexpected wire-breach must SURFACE, never swallow.
      [:my::Counter::GetResponse.RequestTooLarge {:bytes bytes :cap cap}
        (:wat::kernel::assertion-failed! :message "compute: unexpected RequestTooLarge")]
      [:my::Counter::GetResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
        (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::show (:user::compute))))
