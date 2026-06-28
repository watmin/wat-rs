;; typealias_fn_type_spawn.wat — alias over Fn type works at spawn-thread.
(:wat::core::typealias
  :my::Job
  :wat::core::Fn(rust::crossbeam_channel::Sender<wat::core::i64>)->wat::core::nil)
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [job
      (:wat::core::fn [tx <- :wat::kernel::Sender<wat::core::i64>] -> :wat::core::nil
        (:wat::core::do
          (:wat::core::Result/expect (:wat::kernel::send tx 7) "test producer: tx disconnected")
          ()))
     pair
      (:wat::kernel::make-channel :wat::core::i64)
     tx (:wat::core::first pair)
     rx (:wat::core::second pair)
     h
      (:wat::kernel::spawn-thread
        (:wat::core::fn
          [_in <- :wat::kernel::Receiver<wat::core::nil>
           _out <- :wat::kernel::Sender<wat::core::nil>]
           -> :wat::core::nil
          (job tx)))
     _
      (:wat::kernel::Thread/drain-and-join h)]
    (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::i64
      ((:wat::core::Ok (:wat::core::Some v)) v)
      ((:wat::core::Ok :wat::core::None) 0)
      ((:wat::core::Err _died) -1))))
