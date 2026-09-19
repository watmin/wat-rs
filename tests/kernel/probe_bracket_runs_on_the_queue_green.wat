;; Queue path with drop-recv-bp armed. Survives TimedOut (RETRY) and reports
;; the injector's own fires so a green cannot be a vacuous skip.

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
           :record (:wat::queue::queue::Record
                     :cap 1024
                     :store-addr (:wat::query::mem-store::Handle/addr msh)
                     :drop-recv-bp 3000 :drop-ack-bp 0 :drop-seed 20260919))
     addr (:wat::queue::queue::Handle/addr qh)
     q    (:wat::core::match (:wat::kernel::connect addr)
            ((:wat::kernel::ConnectOutcome::Connected p) p)
            ((:wat::kernel::ConnectOutcome::Refused c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Rejected c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Failed c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     got  (:wat::bracket::map
            (:wat::bracket::queue-opts q addr "bdrop" 2 1000000000 8000)
            (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5 6 7 8 9 10)
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
     st   (:wat::core::match
            (:wat::queue::Queue/stats q (:wat::queue::Queue::StatsRequest))
            ((:wat::kernel::RecvOutcome::Message r)
              (:wat::core::match r
                ((:wat::queue::Queue::StatsResponse::Ok s) s)
                (_ (:wat::kernel::assertion-failed! "stats not Ok" :wat::core::None :wat::core::None))))
            (_ (:wat::kernel::assertion-failed! "stats recv failed" :wat::core::None :wat::core::None)))]
    (:wat::core::format
      "result={r};recv-drops={d};recv-replies={n}"
      :r (:wat::string::join ","
           (:wat::core::mapv
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::String
               (:wat::core::format "{x}" :x x))
             got))
      :d (:wat::queue::Stats/recv-drops st)
      :n (:wat::queue::Stats/recv-replies st))))
