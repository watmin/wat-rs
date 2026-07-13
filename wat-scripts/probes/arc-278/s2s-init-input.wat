;; isolate: does an extra :init operating-input (beyond :record) cross a PROCESS fork?
(:wat::core::defsurface :probe::Seedy :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Seedy::GetRequest  [])
   (:wat::core::defrecord :probe::Seedy::GetResponse [v <- :wat::core::i64])]
  :features
  [(get [self <- :probe::Seedy  req <- :probe::Seedy::GetRequest] -> :probe::Seedy::GetResponse)])

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
       (:probe::Seedy::GetResponse :v (:probe::seedy'::State/seed s))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h  (:probe::seedy'/start :locus (:wat::spawn::process) :record (:probe::seedy'::Record) :seed 99)
     c  (:wat::kernel::connect' (:probe::seedy'::Handle/addr h))
     r  (:probe::Seedy/get c (:probe::Seedy::GetRequest))]
    (:wat::kernel::println (:wat::core::i64::to-string (:probe::Seedy::GetResponse/v r)))))
