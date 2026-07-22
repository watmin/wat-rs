;; isolate: does an extra :init operating-input (beyond :record) cross a PROCESS fork?
(:wat::core::defsurface :probe::Seedy :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Seedy::GetRequest  [])
   (:wat::core::defenum :probe::Seedy::GetResponse :wat::enum::Pure :Ok [v <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(get [self <- :probe::Seedy  req <- :probe::Seedy::GetRequest] -> :probe::Seedy::GetResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::seedy'
  :satisfies :probe::Seedy
  :durable   []
  :ephemeral [seed <- :wat::core::i64]
  :init (:wat::core::fn [record <- :probe::seedy'::Record  seed <- :wat::core::i64]
          -> :probe::seedy'::State
          (:probe::seedy'::State :durable record :seed seed))
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Seedy::GetResponse::Ok (:probe::seedy'::State/seed s))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h  (:probe::seedy'/start :locus (:wat::spawn::process) :record (:probe::seedy'::Record) :seed 99)
     c  (:wat::kernel::connect' (:probe::seedy'::Handle/addr h))
     r  (:probe::Seedy/get c (:probe::Seedy::GetRequest))]
    (:wat::kernel::println (:wat::core::i64::to-string (:wat::core::match r -> :wat::core::i64
  ((:probe::Seedy::GetResponse::Ok v) v)
  ((:probe::Seedy::GetResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None)))))))
