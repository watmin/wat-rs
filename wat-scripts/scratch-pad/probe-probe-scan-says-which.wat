;; Drive the probe-scan fold: ScanResponse::Transient (retryable, not a death)
;; and RecvOutcome by dying on scan (put still succeeds).
(:wat::config::set-redef! true)

(:wat::service::defservice :ps::scan-fault
  :satisfies :wat::query::Store
  :durable   [inner-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])
              die?       <- :wat::core::bool]
  :ephemeral [inner <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn [record <- :ps::scan-fault::Record] -> :ps::scan-fault::State
          (:ps::scan-fault::State
            :durable record
            :inner (:wat::core::match
                     (:wat::kernel::connect (:ps::scan-fault::Record/inner-addr record))
                     ((:wat::kernel::ConnectOutcome::Connected p) p)
                     (_ (:wat::kernel::assertion-failed! "ps: inner dial failed"
                          :wat::core::None :wat::core::None)))))
  :impls
  [(ensure-schema [s ctx req]
     (:wat::core::let
       [inner (:ps::scan-fault::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])]
       (:wat::core::match (:wat::query::Store/ensure-schema inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::EnsureSchema r)) sends none))
         (_ (:wat::kernel::assertion-failed! "ps: inner ensure-schema failed"
              :wat::core::None :wat::core::None)))))
   (put [s ctx req]
     (:wat::core::let
       [inner (:ps::scan-fault::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])]
       (:wat::core::match (:wat::query::Store/put inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Put r)) sends none))
         (_ (:wat::kernel::assertion-failed! "ps: inner put failed"
              :wat::core::None :wat::core::None)))))
   (delete [s ctx req]
     (:wat::core::let
       [inner (:ps::scan-fault::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])]
       (:wat::core::match (:wat::query::Store/delete inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::Delete r)) sends none))
         (_ (:wat::kernel::assertion-failed! "ps: inner delete failed"
              :wat::core::None :wat::core::None)))))
   (scan [s ctx req]
     (:wat::core::let
       [die?  (:ps::scan-fault::Record/die? (:ps::scan-fault::State/durable s))
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])
        fault (:wat::query::Fault :message "scan-fault")]
       (:wat::core::if die?
         (:wat::service::Outcome::Stop s :wat::core::None sends)
         (:wat::service::Outcome::Continue s
           (:wat::core::Some (:wat::query::Store::Reply::Scan
             (:wat::query::Store::ScanResponse::Transient
               (:wat::query::Transient :reason fault))))
           sends none))))
   (scan-index [s ctx req]
     (:wat::core::let
       [inner (:ps::scan-fault::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])]
       (:wat::core::match (:wat::query::Store/scan-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::ScanIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "ps: inner scan-index failed"
              :wat::core::None :wat::core::None)))))
   (count-index [s ctx req]
     (:wat::core::let
       [inner (:ps::scan-fault::State/inner s)
        sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none  (:wat::core::Vector :- [(:wat::service::Alarm :- [:ps::scan-fault::Op])])]
       (:wat::core::match (:wat::query::Store/count-index inner req)
         ((:wat::kernel::RecvOutcome::Message r)
           (:wat::service::Outcome::Continue s
             (:wat::core::Some (:wat::query::Store::Reply::CountIndex r)) sends none))
         (_ (:wat::kernel::assertion-failed! "ps: inner count-index failed"
              :wat::core::None :wat::core::None)))))])

(:wat::core::defn :ps::bodies [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::Vector :- [:wat::core::String] "m0"))

(:wat::core::defn :ps::dial-q
  [a <- (:wat::kernel::Address :- [:wat::queue::Queue::Op :wat::queue::Queue::Reply])]
  -> :wat::queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "ps: queue dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :ps::send-tag
  [q <- :wat::queue::Queue] -> :wat::core::String
  (:wat::core::match
    (:wat::queue::Queue/send q
      (:wat::queue::Queue::SendRequest :queue "q" :bodies (:ps::bodies) :now-ns 1000000000))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::queue::Queue::SendResponse::Accepted n)
          (:wat::core::format "Accepted({n})" :n n))
        (_ "other-response")))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::core::format "Lost:{m}" :m (:wat::kernel::LociDiedError/message c)))
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :ps::cell
  [die? <- :wat::core::bool] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     fsh (:ps::scan-fault/start :locus (:wat::spawn::thread)
            :record (:ps::scan-fault::Record
                      :inner-addr (:wat::query::mem-store::Handle/addr msh)
                      :die? die?))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
            :record (:wat::queue::queue::Record
                      :cap 1024 :store-addr (:ps::scan-fault::Handle/addr fsh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:ps::dial-q (:wat::queue::queue::Handle/addr qh))]
    (:ps::send-tag q)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println
         (:wat::core::format "TRANSIENT={s}" :s (:ps::cell false)))
     _ (:wat::kernel::println
         (:wat::core::format "DIE={s}" :s (:ps::cell true)))]
    nil))
