;; tests/process/probe_arc272_rs2_thread_stop_returns_final_state.wat — co-located fixture.
;; arc 278 S4c: the counter's protocol is LIFTED into an explicit surface (:my::Counter) the
;; service WEARS via :satisfies + :impls (the retired :ops clause is gone). counter service on
;; thread locus; stop (per-service, unchanged) returns the final ::Record.
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
    [h     (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record :count 0))
     c     (:wat::core::match (:wat::kernel::connect (:my::counter::Handle/addr h)) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _     (:wat::core::match (:my::Counter/increment c (:my::Counter::IncrementRequest :n 5))
             [:wat::kernel::RecvOutcome.Message {:msg _resp} nil]
             [:wat::kernel::RecvOutcome.Lost {:cause _c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message _c))]
             [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
             [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))
