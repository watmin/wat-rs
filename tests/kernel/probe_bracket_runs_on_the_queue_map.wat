;; Co-located fixture: bracket/map over a queue-opts locus doubles 1..20 in order.

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:wat::queue::queue/start :locus (:wat::spawn::thread)
           :record (:wat::queue::queue::Record
                     :cap 1024
                     :store-addr (:wat::query::mem-store::Handle/addr msh)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     addr (:wat::queue::queue::Handle/addr qh)
     q    (:wat::core::match (:wat::kernel::connect addr)
            ((:wat::kernel::ConnectOutcome::Connected p) p)
            ((:wat::kernel::ConnectOutcome::Refused c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Rejected c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
            ((:wat::kernel::ConnectOutcome::Failed c)
              (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))]
    (:wat::bracket::map
      (:wat::bracket::queue-opts q addr "bmap" 2 1000000000 8000)
      (:wat::core::mapv
        (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i 1))
        (:wat::core::range 0 20))
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))))
