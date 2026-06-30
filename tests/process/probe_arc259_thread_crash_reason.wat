;; tests/process/probe_arc259_thread_crash_reason.wat
;; co-located fixture for probe_arc259_thread_crash_reason.rs
;; startup_beside(file!()) world — thread-peer crash-reason IPC (Arc 259 S3.5a-0).
;;
;; :user::compute spawns a thread peer whose body calls assertion-failed! with a known
;; sentinel. recv' must raise with the sentinel in the error (crash reason travels over pipe).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::thread)
         (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
           (:wat::kernel::assertion-failed! "BOOM-SENTINEL-9173" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::recv' p)]
    0))

