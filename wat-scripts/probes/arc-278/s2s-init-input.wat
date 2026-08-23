;; isolate: does an extra :init operating-input (beyond :record) cross a PROCESS fork?
(:wat::core::defsurface :probe::Seedy :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Seedy::GetRequest  [])
   (:wat::core::defenum :probe::Seedy::GetResponse :wat::enum::Pure :Ok [v <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Seedy  req <- :probe::Seedy::GetRequest] -> :probe::Seedy::GetResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::seedy
  :satisfies :probe::Seedy
  :durable   []
  :ephemeral [seed <- :wat::core::i64]
  :init (:wat::core::fn [record <- :probe::seedy::Record  seed <- :wat::core::i64]
          -> :probe::seedy::State
          (:probe::seedy::State :durable record :seed seed))
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Seedy::GetResponse::Ok (:probe::seedy::State/seed s))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h  (:probe::seedy/start :locus (:wat::spawn::process) :record (:probe::seedy::Record) :seed 99)
     c  (:wat::core::match (:wat::kernel::connect (:probe::seedy::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r  (:probe::Seedy/get c (:probe::Seedy::GetRequest))]
    (:wat::kernel::println (:wat::core::i64::to-string (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Seedy::GetResponse::Ok v) v)
  ((:probe::Seedy::GetResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Seedy::GetResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))))
