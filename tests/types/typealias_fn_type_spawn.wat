;; typealias_fn_type_spawn.wat — alias over Fn type works at spawn-thread.
;;
;; Arc 278 Wave A: migrated off a hand-rolled second channel pair (that was
;; incidental here — the actual subject is the typealias, not a channel pair).
;; `job` now receives the thread's OWN substrate-provided output Sender (the
;; `out <- Sender<i64>` spawn-thread already allocates) instead of a bespoke
;; channel pair; the caller reads it back via `Thread/output`. Same
;; typealias-over-Fn-type-at-spawn-thread subject, zero hand-rolled channels.
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
     h
      (:wat::kernel::spawn-thread
        (:wat::core::fn
          [_in <- :wat::kernel::Receiver<wat::core::nil>
           out <- :wat::kernel::Sender<wat::core::i64>]
           -> :wat::core::nil
          (job out)))
     rx (:wat::kernel::Thread/output h)
     result
      (:wat::core::match (:wat::kernel::recv rx)
        ((:wat::core::Ok (:wat::core::Some v)) v)
        ((:wat::core::Ok :wat::core::None) 0)
        ((:wat::core::Err _died) -1))
     _
      (:wat::kernel::Thread/drain-and-join h)]
    result))
